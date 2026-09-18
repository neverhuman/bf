use crate::digest::sha256_hex;
use crate::gitutil::{commit_all, git, init_repo};
use crate::hub::{Hub, BUGGY_PY, GOOD_PY, PROFILE_JSON, TASK_JSON};
use crate::{Error, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, OptionalExtension, Transaction};
use serde_json::{json, Value};
use std::fs;
use uuid::Uuid;

pub(crate) fn grant(tx: &Transaction, purpose: &str) -> Result<()> {
    let ok:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM grants g JOIN principals p ON p.id=g.subject_id WHERE g.id='demo-grant' AND g.revoked=0 AND p.active=1 AND g.fixture_only=1 AND julianday(g.expires_at)>julianday('now') AND g.used_invocations<g.invocation_allowance AND json_extract(g.capability_json,?1)=1)",[format!("$.{purpose}")],|r|r.get(0))?;
    if !ok {
        return Err(Error::PolicyDenied(format!("current fixture {purpose} grant/allowance required; fixture grants never authorize live execution")));
    }
    Ok(())
}
fn available(tx: &Transaction, amount: i64) -> Result<()> {
    let free:i64=tx.query_row("SELECT limit_micros-actual_micros-(SELECT COALESCE(sum(amount_micros),0) FROM budget_holds WHERE state IN('held','uncertain'))-(SELECT COALESCE(sum(remaining_micros),0) FROM completion_holds WHERE state IN('held','assigned')) FROM budget_accounts WHERE id='demo-usd'",[],|r|r.get(0))?;
    if free < amount {
        return Err(Error::BudgetUnavailable(
            "execution and protected completion reserves exceed remaining budget".into(),
        ));
    }
    Ok(())
}
pub(crate) fn admit_plan(
    tx: &Transaction,
    mission: &str,
    draft: &str,
    revision: i64,
    epoch: &str,
) -> Result<()> {
    let active: bool = tx.query_row(
        "SELECT control_state='active' AND paused=0 FROM missions WHERE id=?1",
        [mission],
        |r| r.get(0),
    )?;
    if !active {
        return Err(Error::PolicyDenied(
            "resume dispatch before replanning; cancelled missions cannot restart".into(),
        ));
    }
    grant(tx, "plan")?;
    available(tx, 1000)?;
    let envelope = json!({"schema_version":3,"draft_id":draft,"revision":revision,"purpose":"plan","fixture_only":true});
    insert_job(
        tx, mission, None, "plan", epoch, revision, &envelope, 1000, None,
    )?;
    Ok(())
}
pub(crate) fn admit_implementation(
    tx: &Transaction,
    mission: &str,
    task: &str,
    epoch: &str,
    plan: &Value,
) -> Result<()> {
    grant(tx, "implement")?;
    let paused: bool = tx.query_row("SELECT paused FROM missions WHERE id=?1", [mission], |r| {
        r.get(0)
    })?;
    if paused {
        return Err(Error::PolicyDenied("mission paused".into()));
    }
    let unmet:i64=tx.query_row("SELECT count(*) FROM dependencies d LEFT JOIN tasks p ON p.id=d.predecessor_id WHERE d.task_id=?1 AND (p.id IS NULL OR p.active_revision<>d.predecessor_revision OR p.phase<>d.condition OR EXISTS(SELECT 1 FROM dependency_invalidations i WHERE i.predecessor_id=p.id AND i.active=1))",[task],|r|r.get(0))?;
    if unmet > 0 {
        return Err(Error::DependencyInvalid(
            "required predecessor is not currently satisfied".into(),
        ));
    }
    let contract: Vec<u8> = tx.query_row(
        "SELECT raw_json FROM contracts WHERE task_id=?1 AND revision=1",
        [task],
        |r| r.get(0),
    )?;
    let contract: Value = serde_json::from_slice(&contract)?;
    // The first fixture has no dependencies. Unknown/nonempty graphs are refused, never dropped.
    if !contract["depends_on"]
        .as_array()
        .is_some_and(|v| v.is_empty())
    {
        return Err(Error::UnknownDependency(
            "fixture planner supports an independent task only".into(),
        ));
    }
    let active: i64 = tx.query_row(
        "SELECT count(*) FROM jobs WHERE task_id=?1 AND writer_authority='active'",
        [task],
        |r| r.get(0),
    )?;
    if active > 0 {
        return Err(Error::ResourceConflict(
            "one current shipping writer".into(),
        ));
    }
    let count: i64 = tx.query_row(
        "SELECT lifetime_invocations FROM tasks WHERE id=?1",
        [task],
        |r| r.get(0),
    )?;
    if count
        >= contract["limits"]["write_invocations"]
            .as_i64()
            .unwrap_or(0)
    {
        return Err(Error::BudgetUnavailable(
            "cumulative write invocation limit".into(),
        ));
    }
    let remaining:i64=tx.query_row("SELECT COALESCE((SELECT remaining_micros FROM completion_holds WHERE task_id=?1 AND state IN('held','assigned')),0)",[task],|r|r.get(0))?;
    let topup = (500000 - remaining).max(0);
    available(tx, 200000 + topup)?;
    let repo = contract["repo_id"]
        .as_str()
        .ok_or_else(|| Error::InvalidContract("repository missing".into()))?;
    let paths = contract["write_paths"]
        .as_array()
        .ok_or_else(|| Error::InvalidContract("scope missing".into()))?;
    let mut q=tx.prepare("SELECT resource_key FROM change_reservations WHERE repo_id=?2 AND active=1 AND task_id<>?1 AND resource_kind='path'")?;
    let held = q
        .query_map(params![task, repo], |r| r.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    for p in paths {
        let path = p
            .as_str()
            .ok_or_else(|| Error::InvalidContract("scope must be strings".into()))?;
        crate::domain::validate_write_path(path)?;
        if held.iter().any(|h| crate::domain::paths_overlap(h, path)) {
            return Err(Error::ResourceConflict(format!("reserved path {path}")));
        }
    }
    // Separate fixture missions have separate repository and reservation identities.
    for p in paths {
        tx.execute("INSERT INTO change_reservations SELECT ?1,?2,?4,'path',?3,1,'owner-demo' WHERE NOT EXISTS(SELECT 1 FROM change_reservations WHERE task_id=?2 AND resource_key=?3 AND active=1)",params![format!("res-{}",Uuid::new_v4()),task,p.as_str(),repo])?;
    }
    let hold = format!("completion-{task}");
    tx.execute("INSERT INTO completion_holds VALUES(?1,?2,500000,500000,'USD','held') ON CONFLICT(id) DO UPDATE SET remaining_micros=500000,state='held',amount_micros=amount_micros+?3",params![hold,task,topup])?;
    let envelope = json!({"schema_version":3,"purpose":"implement","fixture_only":true,"plan":plan,"task_id":task,"accepted_requirements":contract,"remaining_write_invocations":contract["limits"]["write_invocations"].as_i64().unwrap_or(0)-count});
    insert_job(
        tx,
        mission,
        Some(task),
        "implement",
        epoch,
        count + 1,
        &envelope,
        200000,
        None,
    )?;
    tx.execute("UPDATE tasks SET writer_generation=writer_generation+1,lifetime_invocations=lifetime_invocations+1,phase='working',version=version+1 WHERE id=?1",[task])?;
    projection(tx, task, "working", "Accepted; implementation admitted")?;
    Ok(())
}
#[allow(clippy::too_many_arguments)]
fn insert_job(
    tx: &Transaction,
    mission: &str,
    task: Option<&str>,
    purpose: &str,
    epoch: &str,
    generation: i64,
    envelope: &Value,
    amount: i64,
    completion: Option<&str>,
) -> Result<String> {
    let occupied: i64 = tx.query_row(
        "SELECT count(*) FROM jobs WHERE occupancy<>'stopped'",
        [],
        |r| r.get(0),
    )?;
    let capacity: i64 = tx.query_row(
        "SELECT capacity FROM runners WHERE id='runner-fixture' AND enabled=1",
        [],
        |r| r.get(0),
    )?;
    if occupied >= capacity {
        return Err(Error::ResourceConflict(
            "physical runner capacity, including unknown occupancy".into(),
        ));
    }
    let id = format!("J-{}", Uuid::new_v4());
    let incarnation = Uuid::new_v4().to_string();
    let raw = envelope.to_string();
    let check = purpose == "verify";
    tx.execute("INSERT INTO jobs VALUES(?1,?2,?3,?4,?5,'shipping',?6,?7,'runner-fixture',?8,?9,?10,'demo-grant',?11,'prepared',?12,'allocated',?13,?14)",params![id,mission,task,task.map(|_|1),purpose,if check{"check"}else{"model"},if check{None}else{Some(sha256_hex(PROFILE_JSON.as_bytes()))},generation,if purpose=="implement"{Some(generation)}else{None},epoch,(Utc::now()+Duration::minutes(10)).to_rfc3339(),if purpose=="implement"{"active"}else{"none"},sha256_hex(raw.as_bytes()),raw])?;
    tx.execute("INSERT INTO launch_intents(job_id,generation,incarnation,state,workspace) VALUES(?1,?2,?3,'pending',?4)",params![id,generation,incarnation,format!("jobs/{id}/{incarnation}")])?;
    tx.execute(
        "INSERT INTO budget_holds VALUES(?1,?2,'demo-usd',?3,?4,'held')",
        params![format!("budget-{id}"), id, completion, amount],
    )?;
    tx.execute(
        "UPDATE grants SET used_invocations=used_invocations+1 WHERE id='demo-grant'",
        [],
    )?;
    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,job_id,payload_json) VALUES('repo-demo','job_admitted','owner-demo',?1,?2)",params![id,envelope.to_string()])?;
    Ok(id)
}
pub(crate) fn projection(tx: &Transaction, task: &str, phase: &str, why: &str) -> Result<()> {
    tx.execute(
        "UPDATE tasks SET phase=?1,version=version+1 WHERE id=?2",
        params![phase, task],
    )?;
    tx.execute("UPDATE task_projection SET phase=?1,why=?2,version=(SELECT version FROM tasks WHERE id=?3) WHERE task_id=?3",params![phase,why,task])?;
    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,job_id,payload_json) VALUES('repo-demo','task_transition','runner-fixture',(SELECT id FROM jobs WHERE task_id=?1 ORDER BY rowid DESC LIMIT 1),?2)",params![task,json!({"phase":phase,"why":why}).to_string()])?;
    Ok(())
}
#[derive(Clone)]
struct Job {
    id: String,
    mission: String,
    task: Option<String>,
    purpose: String,
    generation: i64,
    envelope: Value,
    workspace: String,
}
impl Hub {
    pub fn drive(&self) -> Result<()> {
        if !self.claim_driver() {
            return Ok(());
        }
        let result = self.drive_inner();
        self.release_driver();
        result
    }
    fn drive_inner(&self) -> Result<()> {
        // One controller turn is bounded. No HTTP handler awaits Git, Python or a remote effect.
        for _ in 0..32 {
            let job=self.db.call(|c|{
                let tx=c.transaction()?;
                let next=tx.query_row("SELECT j.id,j.mission_id,j.task_id,j.purpose,j.assignment_generation,j.envelope_json,l.workspace FROM jobs j JOIN launch_intents l ON l.job_id=j.id JOIN missions m ON m.id=j.mission_id WHERE j.lifecycle='prepared' AND l.state='pending' AND m.paused=0 AND m.control_state='active' ORDER BY CASE j.purpose WHEN 'verify' THEN 0 ELSE 1 END,j.rowid LIMIT 1",[],|r|{let raw:String=r.get(5)?;Ok(Job{id:r.get(0)?,mission:r.get(1)?,task:r.get(2)?,purpose:r.get(3)?,generation:r.get(4)?,envelope:serde_json::from_str(&raw).unwrap_or(Value::Null),workspace:r.get(6)?})}).optional()?;
                if let Some(ref j)=next {
                    // Invocations were consumed at admission, so dispatch checks current grant without a second charge.
                    let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM grants g JOIN principals p ON p.id=g.subject_id JOIN memberships m ON m.principal_id=p.id AND m.repo_id='repo-demo' WHERE g.id='demo-grant' AND g.revoked=0 AND g.fixture_only=1 AND p.active=1 AND m.role IN('administrator','engineer') AND julianday(g.expires_at)>julianday('now') AND json_extract(g.capability_json,?1)=1)",[format!("$.{}",j.purpose)],|r|r.get(0))?;
                    let in_time:bool=tx.query_row("SELECT julianday(deadline_at)>julianday('now') FROM jobs WHERE id=?1",[&j.id],|r|r.get(0))?;
                    let scoped=if let Some(task)=&j.task {
                        let (owner,repo):(String,String)=tx.query_row("SELECT m.owner_id,c.repo_id FROM contracts c JOIN missions m ON m.id=c.mission_id WHERE c.task_id=?1 AND c.revision=1",[task],|r|Ok((r.get(0)?,r.get(1)?)))?;
                        crate::hub::require_membership(&tx,&owner,&repo,true).is_ok()
                    }else{true};
                    if !valid||!in_time||!scoped {
                        reject_prepared(&tx,j,"Grant, deadline or repository access no longer permits dispatch")?;
                        tx.commit()?;return Ok(None);
                    }
                    tx.execute("UPDATE jobs SET lifecycle='starting' WHERE id=?1",[&j.id])?;
                    tx.execute("UPDATE launch_intents SET state='dispatching',checkpoint='before_spawn' WHERE job_id=?1",[&j.id])?;
                    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,job_id,payload_json) VALUES('repo-demo','launch_intent','runner-fixture',?1,'{}')",[&j.id])?;
                }
                tx.commit()?;Ok(next)
            })?;
            let Some(job) = job else { break };
            let flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
            self.cancellations
                .lock()
                .expect("cancellation map")
                .insert(job.id.clone(), flag.clone());
            if let Err(error) = self.running(&job) {
                self.fail_job(&job, &error.to_string())?;
                continue;
            }
            crate::runner::context(Some(flag));
            let result = match job.purpose.as_str() {
                "plan" => self.plan_fixture(&job),
                "implement" => self.implement_fixture(&job),
                "verify" => self.verify_fixture(&job),
                _ => Err(Error::PolicyDenied("unqualified adapter purpose".into())),
            };
            crate::runner::context(None);
            self.cancellations
                .lock()
                .expect("cancellation map")
                .remove(&job.id);
            if let Err(error) = result {
                self.fail_job(&job, &error.to_string())?;
            }
        }
        self.reconcile_effects()?;
        Ok(())
    }
    fn running(&self, job: &Job) -> Result<()> {
        let j = job.clone();
        self.db.call(move|c|{
        let n=c.execute("UPDATE jobs SET lifecycle='running',occupancy='running' WHERE id=?1 AND lifecycle='starting'",[&j.id])?;
        if n!=1{return Err(Error::StaleGeneration)}
        c.execute("UPDATE launch_intents SET state='acknowledged',checkpoint='spawn_observed' WHERE job_id=?1",[&j.id])?;Ok(())})
    }
    fn plan_fixture(&self, job: &Job) -> Result<()> {
        let source = self
            .data_dir
            .join("repos")
            .join(&job.mission)
            .join(format!("plan-{}", job.generation));
        if source.exists() {
            return Err(Error::ResourceConflict(
                "planner source already exists; reconcile retained source before retry".into(),
            ));
        }
        init_repo(&source)?;
        fs::create_dir_all(source.join("src"))?;
        fs::write(source.join("src/dedup.py"), BUGGY_PY)?;
        let (base, _) = commit_all(&source, "fixture baseline")?;
        let mut task: Value = serde_json::from_str(TASK_JSON)?;
        task["id"] = json!(format!("T-{}", job.mission.trim_start_matches("M-")));
        task["mission_id"] = json!(job.mission);
        task["lineage_id"] = task["id"].clone();
        task["repo_id"] = json!(format!("fixture-{}", job.mission));
        git(&source, &["checkout", "-b", "bf/control"])?;
        fs::create_dir_all(source.join("tasks"))?;
        fs::write(
            source
                .join("tasks")
                .join(format!("{}.json", task["id"].as_str().unwrap())),
            serde_json::to_vec(&task)?,
        )?;
        let (control, _) = commit_all(&source, "proposed fixture contract (inactive)")?;
        git(&source, &["checkout", "main"])?;
        let plan = json!({"objective":task["objective"],"acceptance":task["acceptance"],"scope":task["write_paths"],"account":"Deterministic fixture (no account)","model":"fixture-v1","limits":task["limits"],"checks":["dedup-repeat"],"base_oid":base,"control_oid":control,"source":source,"task":task,"fixture_only":true,"review":"Protected fixture review; not live independent review"});
        let j = job.clone();
        self.db.call(move|c|{let tx=c.transaction()?;current(&tx,&j)?;
            let repo=plan["task"]["repo_id"].as_str().unwrap();
            tx.execute("INSERT OR IGNORE INTO repositories VALUES(?1,'fake','{\"fixture_only\":true}')",[repo])?;
            tx.execute("INSERT OR IGNORE INTO memberships SELECT owner_id,?1,'administrator' FROM missions WHERE id=?2",params![repo,j.mission])?;
            let draft=j.envelope["draft_id"].as_str().unwrap_or("");let revision=j.envelope["revision"].as_i64().unwrap_or(0);
            let n=tx.execute("UPDATE drafts SET plan_json=?1,status='proposed' WHERE id=?2 AND revision=?3 AND status='planning'",params![plan.to_string(),draft,revision])?;
            if n!=1{return Err(Error::StaleVersion)}
            crate::commands::message(&tx,draft,"planner","Fixture plan ready: reject repeated delivery IDs. Review the exact scope, checks and limits before Start work.")?;
            finish(&tx,&j,"succeeded")?;tx.commit()?;Ok(())})
    }
    fn implement_fixture(&self, job: &Job) -> Result<()> {
        let plan = &job.envelope["plan"];
        let source = std::path::PathBuf::from(
            plan["source"]
                .as_str()
                .ok_or_else(|| Error::InvalidContract("missing source".into()))?,
        );
        let workspace = self.data_dir.join(&job.workspace);
        fs::create_dir_all(workspace.parent().unwrap())?;
        git(
            &self.data_dir,
            &[
                "clone",
                "--no-local",
                "--no-hardlinks",
                source.to_str().unwrap(),
                workspace.to_str().unwrap(),
            ],
        )?;
        git(&workspace, &["config", "user.name", "BulletFarm fixture"])?;
        git(&workspace, &["config", "user.email", "fixture@bf.local"])?;
        git(
            &workspace,
            &[
                "checkout",
                "--detach",
                plan["base_oid"].as_str().unwrap_or(""),
            ],
        )?;
        fs::write(workspace.join("src/dedup.py"), GOOD_PY)?;
        let (commit, tree) = commit_all(&workspace, "Reject repeated delivery IDs")?;
        let base = plan["base_oid"].as_str().unwrap_or("").to_owned();
        let changes = git(&workspace, &["diff", "--name-status", &base, &commit, "--"])?;
        if changes != "M\tsrc/dedup.py" {
            return Err(Error::PolicyDenied(format!(
                "complete diff outside accepted fixture scope: {changes}"
            )));
        }
        let mode = git(&workspace, &["ls-tree", &commit, "src/dedup.py"])?;
        if !mode.starts_with("100644 blob ") {
            return Err(Error::PolicyDenied("candidate mode changed".into()));
        }
        let source_bytes = git(&workspace, &["show", &format!("{commit}:src/dedup.py")])?;
        if source_bytes.trim_end() != GOOD_PY.trim_end() {
            return Err(Error::InvalidContract(
                "candidate bytes differ from sealed object".into(),
            ));
        }
        let bundle_id = format!("ART-{}", Uuid::new_v4());
        let bundle = self.data_dir.join("artifacts").join(&bundle_id);
        git(
            &workspace,
            &["bundle", "create", bundle.to_str().unwrap(), "HEAD", "main"],
        )?;
        let bytes = fs::read(&bundle)?;
        let digest = sha256_hex(&bytes);
        let size = bytes.len() as i64;
        let candidate = format!("C-{}", Uuid::new_v4());
        let j = job.clone();
        let epoch = self.epoch.clone();
        self.db.call(move|c|{let tx=c.transaction()?;current(&tx,&j)?;let task=j.task.as_deref().unwrap();
            let authority:String=tx.query_row("SELECT writer_authority FROM jobs WHERE id=?1",[&j.id],|r|r.get(0))?;
            if authority!="active"{return Err(Error::StaleGeneration)}
            tx.execute("INSERT INTO artifacts VALUES(?1,'repo-demo',?2,?3,1,?4)",params![bundle_id,digest,size,json!({"kind":"git_bundle","commit":commit,"tree":tree,"base":base}).to_string()])?;
            tx.execute("INSERT INTO candidates VALUES(?1,?2,1,?3,NULL,?4,?5,?6,?7,?8)",params![candidate,task,j.id,commit,tree,base,bundle_id,json!({"fixture_only":true,"full_diff":changes}).to_string()])?;
            let selection:i64=tx.query_row("INSERT INTO selections VALUES(?1,1,?2) ON CONFLICT(task_id) DO UPDATE SET candidate_id=excluded.candidate_id,version=version+1 RETURNING version",params![task,candidate],|r|r.get(0))?;
            tx.execute("UPDATE jobs SET writer_authority='sealed' WHERE id=?1",[&j.id])?;finish(&tx,&j,"succeeded")?;
            projection(&tx,task,"checking","Sealed candidate; protected fixture checks queued")?;
            grant(&tx,"verify")?;
            let hold=format!("completion-{task}");
            let updated=tx.execute("UPDATE completion_holds SET remaining_micros=remaining_micros-100000,state='assigned' WHERE id=?1 AND remaining_micros>=100000",[&hold])?;
            if updated!=1{return Err(Error::BudgetUnavailable("protected verification reserve missing".into()))}
            let envelope=json!({"schema_version":3,"purpose":"verify","fixture_only":true,"candidate_id":candidate,"selection_version":selection,"artifact_id":bundle_id,"artifact_digest":digest,"commit":commit,"tree":tree,"base":base});
            insert_job(&tx,&j.mission,Some(task),"verify",&epoch,1,&envelope,100000,Some(&hold))?;
            tx.commit()?;Ok(())})
    }
    fn verify_fixture(&self, job: &Job) -> Result<()> {
        let artifact = job.envelope["artifact_id"].as_str().unwrap_or("");
        let bundle = self.data_dir.join("artifacts").join(artifact);
        let bytes = fs::read(&bundle)?;
        if sha256_hex(&bytes) != job.envelope["artifact_digest"].as_str().unwrap_or("") {
            return Err(Error::CheckMissing("artifact digest mismatch".into()));
        }
        let workspace = self.data_dir.join(&job.workspace);
        fs::create_dir_all(workspace.parent().unwrap())?;
        git(
            &self.data_dir,
            &[
                "clone",
                "--no-local",
                bundle.to_str().unwrap(),
                workspace.to_str().unwrap(),
            ],
        )?;
        git(
            &workspace,
            &[
                "checkout",
                "--detach",
                job.envelope["commit"].as_str().unwrap_or(""),
            ],
        )?;
        let receipt = crate::verification::verify_fixture(&workspace)?;
        let j = job.clone();
        let receipt_clone = receipt.clone();
        self.db.call(move|c|{let tx=c.transaction()?;current(&tx,&j)?;let task=j.task.as_deref().unwrap();
            let selection:String=tx.query_row("SELECT candidate_id FROM selections WHERE task_id=?1 AND version=?2",params![task,j.envelope["selection_version"].as_i64()],|r|r.get(0))?;
            if selection!=j.envelope["candidate_id"].as_str().unwrap_or(""){return Err(Error::StaleSelection)}
            let producer:String=tx.query_row("SELECT kind FROM principals WHERE id='verifier-fixture' AND active=1",[],|r|r.get(0))?;
            if producer!="verifier"{return Err(Error::PolicyDenied("wrong evidence producer".into()))}
            let result=receipt_clone["result"].as_str().unwrap_or("incomplete");
            let subject=sha256_hex(j.envelope.to_string().as_bytes());
            let evidence=json!({"schema_version":3,"job_id":j.id,"generation":j.generation,"candidate_id":selection,"subject_digest":subject,"inputs":j.envelope,"producer_id":"verifier-fixture","fixture_only":true,"checks":receipt_clone});
            tx.execute("INSERT INTO evidence VALUES(?1,?2,?3,?4,'verifier-fixture','dedup-repeat',?5,?6)",params![format!("EV-{}",Uuid::new_v4()),j.id,selection,subject,result,evidence.to_string()])?;
            finish(&tx,&j,if result=="pass"{"succeeded"}else{"failed"})?;
            if result!="pass" {projection(&tx,task,"failed","Protected fixture check failed")?;}else{crate::delivery::prepare(&tx,task,&selection,&j.envelope)?;}
            tx.commit()?;Ok(())})?;
        Ok(())
    }
    fn fail_job(&self, job: &Job, reason: &str) -> Result<()> {
        let j = job.clone();
        let reason = reason.to_owned();
        self.db.call(move|c|{let tx=c.transaction()?;
        // A synchronous fixture adapter has returned; its child commands have been waited for.
        finish(&tx,&j,"failed")?;
        tx.execute("UPDATE jobs SET writer_authority='revoked' WHERE id=?1 AND writer_authority='active'",[&j.id])?;
        if let Some(task)=&j.task{let cancelled:bool=tx.query_row("SELECT phase='cancelled' FROM tasks WHERE id=?1",[task],|r|r.get(0))?;if !cancelled{projection(&tx,task,"failed",&reason)?;}}else{tx.execute("UPDATE drafts SET status='failed' WHERE mission_id=?1 AND status<>'cancelled'",[&j.mission])?;}
        tx.commit()?;Ok(())})
    }
    pub(crate) fn recover(&self) -> Result<()> {
        self.db.call(|c|{let tx=c.transaction()?;
        // Never infer physical termination or zero usage from a hub restart.
        tx.execute("UPDATE jobs SET lifecycle='unknown',occupancy='unconfirmed',writer_authority=CASE WHEN writer_authority='active' THEN 'revoked' ELSE writer_authority END WHERE lifecycle IN('starting','running','stopping')",[])?;
        tx.execute("UPDATE launch_intents SET state='unknown' WHERE job_id IN(SELECT id FROM jobs WHERE lifecycle='unknown')",[])?;
        tx.execute("UPDATE budget_holds SET state='uncertain' WHERE job_id IN(SELECT id FROM jobs WHERE lifecycle='unknown') AND state='held'",[])?;
        tx.execute("UPDATE task_projection SET why='Interrupted execution; physical occupancy and usage remain unknown' WHERE task_id IN(SELECT task_id FROM jobs WHERE lifecycle='unknown')",[])?;
        tx.execute("UPDATE effects SET state='outcome_unknown' WHERE state='dispatching'",[])?;
        tx.commit()?;Ok(())})?;
        self.reconcile_effects()
    }
}
fn reject_prepared(tx: &Transaction, j: &Job, reason: &str) -> Result<()> {
    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,job_id,payload_json) VALUES('repo-demo','dispatch_rejected','runner-fixture',?1,?2)",params![j.id,json!({"reason":reason,"fixture_only":true}).to_string()])?;
    tx.execute("UPDATE jobs SET lifecycle='interrupted',occupancy='stopped',writer_authority=CASE WHEN writer_authority='active' THEN 'revoked' ELSE writer_authority END WHERE id=?1",[&j.id])?;
    tx.execute(
        "UPDATE launch_intents SET state='stopped' WHERE job_id=?1",
        [&j.id],
    )?;
    tx.execute(
        "UPDATE budget_holds SET state='released' WHERE job_id=?1 AND state='held'",
        [&j.id],
    )?;
    if let Some(task) = &j.task {
        projection(tx, task, "failed", reason)?;
    } else {
        tx.execute(
            "UPDATE drafts SET status='failed' WHERE mission_id=?1 AND status<>'cancelled'",
            [&j.mission],
        )?;
    }
    Ok(())
}
fn current(tx: &Transaction, j: &Job) -> Result<()> {
    let authorized:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM grants g JOIN principals p ON p.id=g.subject_id WHERE g.id='demo-grant' AND g.revoked=0 AND p.active=1 AND julianday(g.expires_at)>julianday('now'))",[],|r|r.get(0))?;
    if !authorized {
        return Err(Error::PolicyDenied(
            "authority changed while execution was in flight".into(),
        ));
    }
    if let Some(task) = &j.task {
        let (owner,repo):(String,String)=tx.query_row("SELECT m.owner_id,c.repo_id FROM contracts c JOIN missions m ON m.id=c.mission_id WHERE c.task_id=?1 AND c.revision=1",[task],|r|Ok((r.get(0)?,r.get(1)?)))?;
        crate::hub::require_membership(tx, &owner, &repo, true)?;
    }
    let ok:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM jobs WHERE id=?1 AND assignment_generation=?2 AND lifecycle='running' AND julianday(deadline_at)>julianday('now'))",params![j.id,j.generation],|r|r.get(0))?;
    if !ok {
        return Err(Error::StaleGeneration);
    }
    Ok(())
}
fn finish(tx: &Transaction, j: &Job, state: &str) -> Result<()> {
    tx.execute(
        "UPDATE jobs SET lifecycle=?1,occupancy='stopped' WHERE id=?2",
        params![state, j.id],
    )?;
    tx.execute("UPDATE launch_intents SET state='stopped',checkpoint='termination_observed' WHERE job_id=?1",[&j.id])?;
    let amount: i64 = tx.query_row(
        "SELECT COALESCE(sum(amount_micros),0) FROM budget_holds WHERE job_id=?1 AND state='held'",
        [&j.id],
        |r| r.get(0),
    )?;
    tx.execute(
        "UPDATE budget_accounts SET actual_micros=actual_micros+?1 WHERE id='demo-usd'",
        [amount],
    )?;
    tx.execute(
        "UPDATE budget_holds SET state='settled' WHERE job_id=?1 AND state='held'",
        [&j.id],
    )?;
    tx.execute("INSERT OR IGNORE INTO usage_observations VALUES(?1,?2,'fixture',0,'cumulative_inclusive',?3,'USD','reported')",params![format!("usage-{}",j.id),j.id,amount])?;
    tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,job_id,payload_json) VALUES('repo-demo','termination_observed','runner-fixture',?1,?2)",params![j.id,json!({"lifecycle":state,"fixture_only":true}).to_string()])?;
    Ok(())
}
