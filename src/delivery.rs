use crate::digest::sha256_hex;
use crate::forge::FakeForge;
use crate::hub::Hub;
use crate::{Error, Result};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) fn prepare(tx: &Transaction, task: &str, candidate: &str, inputs: &Value) -> Result<()> {
    let key = format!("pr:{task}:1");
    let payload = json!({"fixture_only":true,"destination":"fake","logical_key":key,"candidate_id":candidate,"task_revision":1,"selection_version":inputs["selection_version"],"head":inputs["commit"],"base":inputs["base"],"expected_remote":"absent_or_exact_marker_and_head","check_job_inputs":inputs,"title":"Reject repeated delivery IDs"});
    let raw = payload.to_string();
    tx.execute("INSERT INTO effects VALUES(?1,?2,?3,?4,?5,?7,'demo-grant','publisher-fixture','pending',?6,NULL)",params![format!("E-{}",Uuid::new_v4()),key,sha256_hex(raw.as_bytes()),task,candidate,raw,inputs["selection_version"].as_i64()])?;
    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,payload_json) VALUES('repo-demo','publication_intent','publisher-fixture',?1)",[payload.to_string()])?;
    Ok(())
}
#[derive(Clone)]
struct Effect {
    id: String,
    key: String,
    task: String,
    candidate: String,
    state: String,
    payload: Value,
}
impl Hub {
    pub fn set_fixture_fault(&self, point: &str) -> Result<()> {
        if !matches!(
            point,
            "before_dispatch"
                | "after_dispatch"
                | "after_remote"
                | "after_receipt"
                | "before_settlement"
        ) {
            return Err(Error::InvalidContract("unknown fixture fault".into()));
        }
        let point = point.to_owned();
        self.db.call(move |c| {
            c.execute(
                "INSERT OR REPLACE INTO fixture_faults VALUES(?1,1)",
                [point],
            )?;
            Ok(())
        })
    }
    fn fault(&self, point: &str) -> Result<()> {
        let point = point.to_owned();
        let name = point.clone();
        let fired = self.db.call(move |c| {
            let n = c.execute(
                "UPDATE fixture_faults SET remaining=0 WHERE point=?1 AND remaining=1",
                [point],
            )?;
            Ok(n == 1)
        })?;
        if fired {
            Err(Error::OutcomeUnknown(format!(
                "injected fixture crash: {name}"
            )))
        } else {
            Ok(())
        }
    }
    pub fn reconcile_effects(&self) -> Result<()> {
        let effects=self.db.call(|c|{let mut q=c.prepare("SELECT e.id,e.logical_key,e.task_id,e.candidate_id,e.state,e.payload_json FROM effects e JOIN tasks t ON t.id=e.task_id WHERE e.state IN('pending','dispatching','outcome_unknown','confirmed') AND t.phase<>'review_ready' ORDER BY e.rowid LIMIT 32")?;
            let rows=q.query_map([],|r|{let raw:String=r.get(5)?;Ok(Effect{id:r.get(0)?,key:r.get(1)?,task:r.get(2)?,candidate:r.get(3)?,state:r.get(4)?,payload:serde_json::from_str(&raw).unwrap_or(Value::Null)})})?;Ok(rows.collect::<std::result::Result<Vec<_>,_>>()?)})?;
        for effect in effects {
            if effect.payload["fixture_only"] != true || effect.payload["destination"] != "fake" {
                continue;
            }
            let e = effect.clone();
            let artifact=self.db.call(move|c|{Ok(c.query_row("SELECT a.id,a.digest,a.byte_count FROM artifacts a JOIN candidates ca ON ca.artifact_id=a.id WHERE ca.id=?1 AND a.complete=1",[e.candidate],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))).optional()?)})?;
            let artifact_valid = artifact.is_some_and(|(id, digest, size)| {
                std::fs::read(self.data_dir.join("artifacts").join(id))
                    .is_ok_and(|b| b.len() as i64 == size && sha256_hex(&b) == digest)
            });
            let forge = FakeForge::open(&self.data_dir.join("fake-forge"))?;
            // Read-back can record an already-applied outcome even after cancellation/revocation.
            let observed = forge.find(&effect.key)?;
            let pr = if let Some(pr) = observed {
                pr
            } else {
                if !artifact_valid {
                    let id = effect.id.clone();
                    self.db.call(move |c| {
                        c.execute("UPDATE effects SET state='blocked' WHERE id=?1", [id])?;
                        Ok(())
                    })?;
                    continue;
                }
                self.fault("before_dispatch")?;
                let e = effect.clone();
                let admitted=self.db.call(move|c|{let tx=c.transaction()?;let allowed=eligible(&tx,&e)?;
                    if allowed {tx.execute("UPDATE effects SET state='dispatching' WHERE id=?1",[&e.id])?;}else{tx.execute("UPDATE task_projection SET why='Publication held: current grant, selection, checks or mission authorization changed' WHERE task_id=?1",[&e.task])?;}
                    tx.commit()?;Ok(allowed)})?;
                if !admitted {
                    continue;
                }
                self.fault("after_dispatch")?;
                let pr = forge.create(
                    &effect.key,
                    effect.payload["head"].as_str().unwrap_or(""),
                    effect.payload["base"].as_str().unwrap_or(""),
                    effect.payload["title"].as_str().unwrap_or(""),
                )?;
                self.fault("after_remote")?;
                pr
            };
            if pr.human_closed
                || Some(pr.head.as_str()) != effect.payload["head"].as_str()
                || Some(pr.base.as_str()) != effect.payload["base"].as_str()
            {
                let id = effect.id.clone();
                self.db.call(move |c| {
                    c.execute("UPDATE effects SET state='blocked' WHERE id=?1", [id])?;
                    Ok(())
                })?;
                continue;
            }
            let e = effect.clone();
            let receipt = json!({"number":pr.number,"url":pr.url,"head":pr.head,"base":pr.base,"logical_key":pr.logical_key,"draft":true,"fixture_only":true,"reconciled":effect.state!="pending"});
            let recorded = receipt.clone();
            self.db.call(move |c| {
                c.execute(
                    "UPDATE effects SET state='confirmed',receipt_json=?1 WHERE id=?2",
                    params![recorded.to_string(), e.id],
                )?;
                Ok(())
            })?;
            self.fault("after_receipt")?;
            self.fault("before_settlement")?;
            let e = effect;
            self.db.call(move|c|{let tx=c.transaction()?;
                if artifact_valid && eligible(&tx,&e)? {
                    crate::jobs::projection(&tx,&e.task,"review_ready","Fixture checks complete; simulated draft PR ready (not live qualification)")?;
                    tx.execute("UPDATE task_projection SET pr_json=?1 WHERE task_id=?2",params![receipt.to_string(),e.task])?;
                    tx.execute("UPDATE completion_holds SET state='released' WHERE task_id=?1",[&e.task])?;
                }
                tx.commit()?;Ok(())})?;
        }
        Ok(())
    }
}
fn eligible(tx: &Transaction, e: &Effect) -> Result<bool> {
    let selection: Option<String> = tx
        .query_row(
            "SELECT candidate_id FROM selections WHERE task_id=?1 AND version=?2",
            params![e.task, e.payload["selection_version"].as_i64()],
            |r| r.get(0),
        )
        .optional()?;
    if selection.as_deref() != Some(&e.candidate) {
        return Ok(false);
    }
    let permitted:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM tasks t JOIN contracts c ON c.task_id=t.id AND c.revision=t.active_revision JOIN missions m ON m.id=c.mission_id JOIN grants g ON g.id='demo-grant' JOIN principals p ON p.id='publisher-fixture' WHERE t.id=?1 AND t.active_revision=1 AND t.phase='checking' AND m.paused=0 AND m.control_state='active' AND EXISTS(SELECT 1 FROM principals owner JOIN memberships access ON access.principal_id=owner.id WHERE owner.id=m.owner_id AND owner.active=1 AND access.repo_id=c.repo_id AND access.role IN('administrator','engineer')) AND g.fixture_only=1 AND g.revoked=0 AND julianday(g.expires_at)>julianday('now') AND json_extract(g.capability_json,'$.publish')=1 AND p.active=1 AND p.kind='publisher' AND EXISTS(SELECT 1 FROM change_reservations r WHERE r.task_id=t.id AND r.active=1))",[&e.task],|r|r.get(0))?;
    if !permitted {
        return Ok(false);
    }
    let evidence:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM evidence ev JOIN jobs j ON j.id=ev.job_id JOIN candidates c ON c.id=ev.candidate_id JOIN artifacts a ON a.id=c.artifact_id JOIN principals p ON p.id=ev.producer_id WHERE ev.candidate_id=?1 AND ev.check_id='dedup-repeat' AND ev.result='pass' AND ev.producer_id='verifier-fixture' AND p.active=1 AND p.kind='verifier' AND j.purpose='verify' AND j.lifecycle='succeeded' AND j.task_id=c.task_id AND j.task_revision=c.task_revision AND a.complete=1 AND json_extract(j.envelope_json,'$.candidate_id')=c.id AND json_extract(j.envelope_json,'$.commit')=c.commit_oid AND json_extract(j.envelope_json,'$.selection_version')=?2)",params![e.candidate,e.payload["selection_version"].as_i64()],|r|r.get(0))?;
    Ok(evidence)
}
