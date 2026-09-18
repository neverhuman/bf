use crate::digest::{parse_strict_json, require_utf8, sha256_hex};
use crate::hub::{active_actor, require_membership, Hub, Operation};
use crate::{Error, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension, Transaction};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
struct Command {
    schema_version: u32,
    command_id: String,
    kind: CommandKind,
    #[serde(default)]
    target_id: Option<String>,
    #[serde(default)]
    #[ts(type = "number | null")]
    expected_version: Option<i64>,
    #[ts(type = "unknown")]
    payload: Value,
}
#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
enum CommandKind {
    Run,
    RememberProject,
    EditDraft,
    StartWork,
    RetryTask,
    CreateMission,
    Pause,
    Stop,
    Cancel,
    Resume,
    Take,
    SubmitHuman,
    GrantAllowance,
    ResolveDecision,
}
impl CommandKind {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Run => "run",
            Self::RememberProject => "remember_project",
            Self::EditDraft => "edit_draft",
            Self::StartWork => "start_work",
            Self::RetryTask => "retry_task",
            Self::CreateMission => "create_mission",
            Self::Pause => "pause",
            Self::Stop => "stop",
            Self::Cancel => "cancel",
            Self::Resume => "resume",
            Self::Take => "take",
            Self::SubmitHuman => "submit_human",
            Self::GrantAllowance => "grant_allowance",
            Self::ResolveDecision => "resolve_decision",
        }
    }
}
#[derive(Deserialize, ts_rs::TS)]
#[serde(deny_unknown_fields)]
struct Goal {
    goal: String,
    #[serde(default = "demo_project")]
    project_id: String,
}
pub(crate) fn typescript() -> String {
    format!(
        "// Generated from Rust; run scripts/generate-contracts.\n{}{}{}",
        crate::protocol::declaration::<CommandKind>(),
        crate::protocol::declaration::<Command>(),
        crate::protocol::declaration::<Goal>()
    )
}
fn demo_project() -> String {
    "demo".into()
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mission {
    #[serde(default)]
    id: Option<String>,
    goal: String,
    #[serde(default)]
    owner_id: Option<String>,
}
fn empty_payload(value: &Value) -> Result<()> {
    if value.as_object().is_some_and(|m| m.is_empty()) {
        Ok(())
    } else {
        Err(Error::InvalidContract(
            "this command requires an empty payload".into(),
        ))
    }
}
fn text(value: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.len() > max || value.contains('\0') {
        Err(Error::InvalidContract(
            "text is empty, too long, or contains NUL".into(),
        ))
    } else {
        Ok(())
    }
}
impl Hub {
    pub fn command_bytes(&self, actor: &str, raw: &[u8]) -> Result<Operation> {
        self.command_inner(actor, raw, None)
    }
    pub fn command_session(&self, token: &str, raw: &[u8]) -> Result<Operation> {
        let actor = self.lookup_session(token)?;
        self.command_inner(&actor, raw, Some(sha256_hex(token.as_bytes())))
    }
    fn command_inner(&self, actor: &str, raw: &[u8], session: Option<String>) -> Result<Operation> {
        if raw.len() > 65536 {
            return Err(Error::InvalidContract("command exceeds 64 KiB".into()));
        }
        let value = parse_strict_json(require_utf8(raw)?)?;
        let command: Command = serde_json::from_value(value)?;
        if command.schema_version != 3 {
            return Err(Error::InvalidContract("schema_version must be 3".into()));
        }
        text(&command.command_id, 128)?;
        let actor = actor.to_owned();
        let raw = raw.to_vec();
        let epoch = self.epoch.clone();
        let kind = command.kind.as_str();
        let operation=self.db.call(move |c| {
            let tx=c.transaction()?;
            if let Some(session)=session {let valid:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM sessions WHERE token=?1 AND principal_id=?2 AND revoked=0 AND julianday(expires_at)>julianday('now'))",params![session,actor],|r|r.get(0))?;if !valid{return Err(Error::AuthRequired)}}
            let actor_kind=active_actor(&tx,&actor)?;
            require_membership(&tx,&actor,"repo-demo",true)?;
            if actor_kind!="human" {return Err(Error::PolicyDenied("interactive human command required; a model acting for an owner is still an agent".into()))}
            // Authorization precedes historical replay. Raw bytes, including whitespace, are identity.
            if let Some((digest,op_id))=tx.query_row("SELECT request_digest,operation_id FROM commands WHERE actor_id=?1 AND command_id=?2",params![actor,command.command_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?))).optional()? {
                if digest!=sha256_hex(&raw) {return Err(Error::CommandConflict)}
                authorize_target(&tx,&actor,command.target_id.as_deref(),false)?;
                let result:String=tx.query_row("SELECT result_json FROM operations WHERE id=?1",[&op_id],|r|r.get(0))?;
                let result:Value=serde_json::from_str(&result)?;authorize_result(&tx,&actor,&result)?;
                return Ok(Operation{id:op_id,status:"replayed".into(),result})
            }
            authorize_target(&tx,&actor,command.target_id.as_deref(),true)?;
            let result=match command.kind.as_str() {
                "remember_project"=> {
                    #[derive(Deserialize)] #[serde(deny_unknown_fields)]
                    struct Project {path:String,base_oid:String,name:String}
                    let p:Project=serde_json::from_value(command.payload)?;
                    text(&p.path,4096)?;text(&p.name,256)?;
                    if !std::path::Path::new(&p.path).is_absolute()||!matches!(p.base_oid.len(),40|64)||!p.base_oid.bytes().all(|b|b.is_ascii_hexdigit()){return Err(Error::InvalidContract("absolute repository path and exact Git base required".into()))}
                    let id=format!("project-{}",&sha256_hex(p.path.as_bytes())[..16]);
                    tx.execute("INSERT INTO projects VALUES(?1,?2,?3,?4,?5,'live',?6) ON CONFLICT(owner_id,path) DO UPDATE SET last_opened=excluded.last_opened,base_oid=excluded.base_oid",params![id,actor,p.name,p.path,p.base_oid,Utc::now().to_rfc3339()])?;
                    json!({"project_id":id,"status":"remembered","qualified":false})
                }
                "create_mission"=> {
                    let payload:Mission=serde_json::from_value(command.payload)?;
                    if payload.owner_id.as_ref().is_some_and(|o|o!=&actor){return Err(Error::PolicyDenied("body owner fields never authenticate".into()))}
                    if !matches!(payload.goal.as_str(),"pr_ready"|"merged"|"production_observed"|"evidence_accepted"){return Err(Error::InvalidContract("invalid delivery goal".into()))}
                    let id=payload.id.unwrap_or_else(||format!("M-{}",Uuid::new_v4()));text(&id,128)?;
                    tx.execute("INSERT INTO missions(id,owner_id,domain_id,goal,version,paused,contract_json) VALUES(?1,?2,'core',?3,1,0,'{}')",params![id,actor,payload.goal])?;
                    json!({"mission_id":id,"version":1,"goal":payload.goal})
                }
                "run"=> {
                    if command.target_id.is_some()||command.expected_version.is_some(){return Err(Error::InvalidContract("new goal cannot target an existing revision".into()))}
                    let payload:Goal=serde_json::from_value(command.payload)?; text(&payload.goal,16000)?;
                    let mode:String=tx.query_row("SELECT mode FROM projects WHERE id=?1 AND owner_id=?2",params![payload.project_id,actor],|r|r.get(0)).optional()?.ok_or(Error::AuthRequired)?;
                    let mission=format!("M-{}",Uuid::new_v4());let draft=format!("D-{}",Uuid::new_v4());
                    tx.execute("INSERT INTO missions(id,owner_id,domain_id,goal,version,paused,contract_json) VALUES(?1,?2,'core','pr_ready',1,0,?3)",params![mission,actor,json!({"title":payload.goal}).to_string()])?;
                    let status=if mode=="fixture"{"planning"}else{"blocked"};
                    tx.execute("INSERT INTO drafts VALUES(?1,?2,?3,?4,1,?5,NULL,NULL,?6)",params![draft,mission,actor,payload.project_id,payload.goal,status])?;
                    message(&tx,&draft,"owner",&payload.goal)?;
                    if mode=="fixture" { crate::jobs::admit_plan(&tx,&mission,&draft,1,&epoch)?; }
                    else {message(&tx,&draft,"controller","Goal saved. Live planning is blocked by Operating HOLD and missing runner, Codex and grant qualification.")?;}
                    json!({"draft_id":draft,"mission_id":mission,"revision":1,"status":status})
                }
                "edit_draft"=> {
                    let payload:Goal=serde_json::from_value(command.payload)?;text(&payload.goal,16000)?;
                    let id=target(&command.target_id)?;
                    let (mission,revision,status,project)=owned_draft(&tx,&actor,id)?;
                    if Some(revision)!=command.expected_version||!matches!(status.as_str(),"proposed"|"blocked"|"failed"){return Err(Error::StaleVersion)}
                    if project!=payload.project_id{return Err(Error::InvalidContract("changing the project requires a new goal".into()))}
                    let mode:String=tx.query_row("SELECT mode FROM projects WHERE id=?1",[&project],|r|r.get(0))?;
                    let status=if mode=="fixture"{"planning"}else{"blocked"};
                    tx.execute("UPDATE drafts SET goal=?1,revision=revision+1,plan_json=NULL,status=?2 WHERE id=?3",params![payload.goal,status,id])?;
                    message(&tx,id,"owner",&payload.goal)?;
                    if mode=="fixture" {crate::jobs::admit_plan(&tx,&mission,id,revision+1,&epoch)?;}
                    json!({"draft_id":id,"revision":revision+1,"status":status})
                }
                "start_work"=> {
                    empty_payload(&command.payload)?;
                    let id=target(&command.target_id)?;
                    let (mission,revision,status,_)=owned_draft(&tx,&actor,id)?;
                    if Some(revision)!=command.expected_version||status!="proposed" {return Err(Error::StaleVersion)}
                    let plan:String=tx.query_row("SELECT plan_json FROM drafts WHERE id=?1",[id],|r|r.get(0))?;
                    let plan:Value=serde_json::from_str(&plan)?;
                    let task=plan["task"].clone();crate::domain::validate_task(&task)?;
                    let task_id=task["id"].as_str().ok_or_else(||Error::InvalidContract("plan task missing".into()))?;
                    let raw=serde_json::to_vec(&task)?;
                    tx.execute("INSERT INTO contracts VALUES(?1,1,?2,?6,?3,'sha256_exact_utf8_v3',?4,?5)",params![task_id,mission,sha256_hex(&raw),plan["control_oid"].as_str().unwrap_or(""),raw,task["repo_id"].as_str()])?;
                    tx.execute("INSERT INTO tasks(id,active_revision,version,phase,lineage_id) VALUES(?1,1,1,'ready',?1)",[task_id])?;
                    tx.execute("INSERT INTO task_projection(task_id,owner_id,repo_id,mission_id,title,phase,why,version) VALUES(?1,?2,?5,?3,?4,'ready','Accepted plan; waiting for runner',1)",params![task_id,actor,mission,task["title"].as_str().unwrap_or(task_id),task["repo_id"].as_str()])?;
                    crate::jobs::admit_implementation(&tx,&mission,task_id,&epoch,&plan)?;
                    tx.execute("UPDATE drafts SET status='accepted',accepted_revision=revision WHERE id=?1",[id])?;
                    message(&tx,id,"controller","Exact plan accepted; execution and its protected verification reserve admitted atomically.")?;
                    json!({"draft_id":id,"mission_id":mission,"task_id":task_id,"revision":revision,"status":"accepted"})
                }
                "retry_task"=> {
                    empty_payload(&command.payload)?;
                    let task=target(&command.target_id)?;
                    let (mission,version,phase,plan):(String,i64,String,String)=tx.query_row("SELECT c.mission_id,t.version,t.phase,d.plan_json FROM tasks t JOIN contracts c ON c.task_id=t.id AND c.revision=t.active_revision JOIN drafts d ON d.mission_id=c.mission_id WHERE t.id=?1 AND d.owner_id=?2",params![task,actor],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?.ok_or(Error::AuthRequired)?;
                    if Some(version)!=command.expected_version||phase!="failed"{return Err(Error::StaleVersion)}
                    let exposure:i64=tx.query_row("SELECT (SELECT count(*) FROM jobs WHERE task_id=?1 AND occupancy<>'stopped')+(SELECT count(*) FROM effects WHERE task_id=?1)",[task],|r|r.get(0))?;
                    if exposure!=0{return Err(Error::PolicyDenied("reconcile physical occupancy, usage and existing publication before retry".into()))}
                    let plan:Value=serde_json::from_str(&plan)?;
                    crate::jobs::admit_implementation(&tx,&mission,task,&epoch,&plan)?;
                    tx.execute("UPDATE selections SET candidate_id=NULL,version=version+1 WHERE task_id=?1",[task])?;
                    json!({"task_id":task,"mission_id":mission,"status":"retry_admitted"})
                }
                "take"|"submit_human"=>return Err(Error::PolicyDenied("Human takeover/return is not implemented; no workspace ownership has changed. Use Stop to revoke execution.".into())),
                "pause"|"stop"|"cancel"|"resume"=> {
                    empty_payload(&command.payload)?;
                    let id=target(&command.target_id)?;
                    let version:i64=tx.query_row("SELECT version FROM missions WHERE id=?1 AND owner_id=?2",params![id,actor],|r|r.get(0)).optional()?.ok_or(Error::AuthRequired)?;
                    if Some(version)!=command.expected_version{return Err(Error::StaleVersion)}
                    let affected_jobs={let mut q=tx.prepare("SELECT id FROM jobs WHERE mission_id=?1 AND lifecycle IN('prepared','starting','running')")?;let rows=q.query_map([id],|r|r.get::<_,String>(0))?;rows.collect::<std::result::Result<Vec<_>,_>>()?};
                    let resume=command.kind.as_str()=="resume";
                    let control:String=tx.query_row("SELECT control_state FROM missions WHERE id=?1",[id],|r|r.get(0))?;
                    if control=="cancelled"{return Err(Error::PolicyDenied("cancelled missions cannot resume or dispatch".into()))}
                    if command.kind.as_str()=="cancel" {
                        tx.execute("UPDATE missions SET control_state='cancelled' WHERE id=?1",[id])?;
                        tx.execute("UPDATE tasks SET phase='cancelled',version=version+1 WHERE id IN(SELECT task_id FROM contracts WHERE mission_id=?1)",[id])?;
                        tx.execute("UPDATE task_projection SET phase='cancelled',why='Cancelled; existing artifacts and remote observations retained' WHERE mission_id=?1",[id])?;
                        tx.execute("UPDATE drafts SET status='cancelled' WHERE mission_id=?1",[id])?;
                    }
                    tx.execute("UPDATE missions SET paused=?1,version=version+1 WHERE id=?2",params![!resume,id])?;
                    if matches!(command.kind.as_str(),"stop"|"cancel") {
                        tx.execute("UPDATE drafts SET status='failed' WHERE mission_id=?1 AND status='planning' AND EXISTS(SELECT 1 FROM jobs WHERE mission_id=?1 AND purpose='plan' AND lifecycle='prepared')",[id])?;
                        tx.execute("UPDATE tasks SET phase='failed',version=version+1 WHERE phase<>'cancelled' AND id IN(SELECT task_id FROM jobs WHERE mission_id=?1 AND lifecycle='prepared' AND purpose IN('implement','verify'))",[id])?;
                        tx.execute("UPDATE task_projection SET phase='failed',why='Stopped before dispatch; resume and retry within plan limits',version=(SELECT version FROM tasks WHERE id=task_projection.task_id) WHERE phase<>'cancelled' AND task_id IN(SELECT task_id FROM jobs WHERE mission_id=?1 AND lifecycle='prepared' AND purpose IN('implement','verify'))",[id])?;
                        tx.execute("UPDATE budget_holds SET state='released' WHERE state='held' AND job_id IN(SELECT id FROM jobs WHERE mission_id=?1 AND lifecycle='prepared')",[id])?;
                        tx.execute("UPDATE completion_holds SET state='released' WHERE task_id IN(SELECT task_id FROM jobs WHERE mission_id=?1 AND lifecycle='prepared' AND purpose='implement')",[id])?;
                        tx.execute("UPDATE jobs SET lifecycle=CASE WHEN lifecycle='prepared' THEN 'interrupted' ELSE 'stopping' END,writer_authority=CASE WHEN writer_authority='active' THEN 'revoked' ELSE writer_authority END,occupancy=CASE WHEN lifecycle='prepared' THEN 'stopped' ELSE occupancy END WHERE mission_id=?1 AND lifecycle IN('prepared','starting','running')",[id])?;
                        tx.execute("UPDATE launch_intents SET state='stopped' WHERE job_id IN(SELECT id FROM jobs WHERE mission_id=?1 AND occupancy='stopped')",[id])?;
                        tx.execute("UPDATE task_projection SET why='Stop requested; waiting for observed termination' WHERE mission_id=?1 AND phase NOT IN('failed','cancelled')",[id])?;
                    }
                    let remaining:i64=tx.query_row("SELECT COUNT(*) FROM jobs WHERE mission_id=?1 AND occupancy<>'stopped'",[id],|r|r.get(0))?;
                    json!({"mission_id":id,"version":version+1,"status":if resume{"resumed"}else if command.kind.as_str()=="pause"{"paused"}else if remaining>0{"stopping"}else{"stopped"},"occupied_jobs":remaining,"affected_job_ids":affected_jobs})
                }
                other=>return Err(Error::InvalidContract(format!("unsupported command {other}")))
            };
            let id=format!("op-{}",Uuid::new_v4());
            tx.execute("INSERT INTO commands VALUES(?1,?2,?3,?4,?5)",params![actor,command.command_id,sha256_hex(&raw),id,require_utf8(&raw)?])?;
            tx.execute("INSERT INTO operations VALUES(?1,?2,?3,'accepted',?4)",params![id,actor,command.command_id,result.to_string()])?;
            tx.execute("INSERT INTO events(scope_repo_id,kind,producer_id,payload_json) VALUES('repo-demo','command',?1,?2)",params![actor,result.to_string()])?;
            tx.commit()?; Ok(Operation{id,status:"accepted".into(),result})
        })?;
        if operation.status == "accepted" && matches!(kind, "stop" | "cancel") {
            if let Some(jobs) = operation.result["affected_job_ids"].as_array() {
                let flags = self.cancellations.lock().expect("cancellation map");
                for id in jobs.iter().filter_map(Value::as_str) {
                    if let Some(flag) = flags.get(id) {
                        flag.store(true, std::sync::atomic::Ordering::Release);
                    }
                }
            }
        }
        Ok(operation)
    }
    pub fn projects(&self, actor: &str) -> Result<Value> {
        let actor = actor.to_owned();
        self.db.call(move|c|{
        active_actor(c,&actor)?;require_membership(c,&actor,"repo-demo",false)?;
        let mut q=c.prepare("SELECT id,name,path,base_oid,mode FROM projects WHERE owner_id=?1 ORDER BY last_opened DESC,id")?;
        let rows=q.query_map([actor],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"path":r.get::<_,String>(2)?,"base_oid":r.get::<_,String>(3)?,"mode":r.get::<_,String>(4)?})))?;
        Ok(json!({"items":rows.collect::<std::result::Result<Vec<_>,_>>()?}))
    })
    }
    pub fn drafts(&self, actor: &str) -> Result<Value> {
        let actor = actor.to_owned();
        self.db.call(move|c|{
        active_actor(c,&actor)?;require_membership(c,&actor,"repo-demo",false)?;
        let mut q=c.prepare("SELECT d.id,d.mission_id,d.project_id,d.revision,d.goal,d.plan_json,d.status,m.version,m.paused FROM drafts d JOIN missions m ON m.id=d.mission_id WHERE d.owner_id=?1 AND (d.plan_json IS NULL OR EXISTS(SELECT 1 FROM memberships access WHERE access.principal_id=?1 AND access.repo_id=json_extract(d.plan_json,'$.task.repo_id'))) ORDER BY d.rowid DESC LIMIT 100")?;
        let rows=q.query_map([actor],|r|{let raw:Option<String>=r.get(5)?;Ok(json!({"id":r.get::<_,String>(0)?,"mission_id":r.get::<_,String>(1)?,"project_id":r.get::<_,String>(2)?,"revision":r.get::<_,i64>(3)?,"goal":r.get::<_,String>(4)?,"plan":raw.and_then(|s|serde_json::from_str::<Value>(&s).ok()),"status":r.get::<_,String>(6)?,"mission_version":r.get::<_,i64>(7)?,"paused":r.get::<_,bool>(8)?}))})?;
        Ok(json!({"items":rows.collect::<std::result::Result<Vec<_>,_>>()?}))
    })
    }
    pub fn detail(&self, actor: &str, draft: &str) -> Result<Value> {
        let actor = actor.to_owned();
        let draft = draft.to_owned();
        self.db.call(move|c|{
        active_actor(c,&actor)?;require_membership(c,&actor,"repo-demo",false)?;
        let mission:String=c.query_row("SELECT mission_id FROM drafts WHERE id=?1 AND owner_id=?2",params![draft,actor],|r|r.get(0)).optional()?.ok_or(Error::AuthRequired)?;
        authorize_result(c,&actor,&json!({"mission_id":mission}))?;
        let mut q=c.prepare("SELECT seq,role,body FROM conversation WHERE draft_id=?1 ORDER BY seq LIMIT 200")?;
        let conversation=q.query_map([draft],|r|Ok(json!({"seq":r.get::<_,i64>(0)?,"role":r.get::<_,String>(1)?,"body":r.get::<_,String>(2)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
        let mut q=c.prepare("SELECT id,purpose,lifecycle,occupancy,writer_authority,assignment_generation FROM jobs WHERE mission_id=?1 ORDER BY rowid LIMIT 100")?;
        let jobs=q.query_map([mission],|r|Ok(json!({"id":r.get::<_,String>(0)?,"purpose":r.get::<_,String>(1)?,"lifecycle":r.get::<_,String>(2)?,"occupancy":r.get::<_,String>(3)?,"authority":r.get::<_,String>(4)?,"generation":r.get::<_,i64>(5)?})))?.collect::<std::result::Result<Vec<_>,_>>()?;
        Ok(json!({"conversation":conversation,"jobs":jobs}))
    })
    }
    pub fn register_project(&self, actor: &str, path: &std::path::Path) -> Result<String> {
        // Called by the local launcher, outside HTTP handlers. No keys or provider calls.
        let root = crate::gitutil::git(path, &["rev-parse", "--show-toplevel"])?;
        let base = crate::gitutil::git(path, &["rev-parse", "HEAD"])?;
        let actor = actor.to_owned();
        let id = format!("project-{}", &sha256_hex(root.as_bytes())[..16]);
        let result = id.clone();
        self.db.call(move|c|{active_actor(c,&actor)?;let name=std::path::Path::new(&root).file_name().unwrap_or_default().to_string_lossy().to_string();
            c.execute("INSERT INTO projects VALUES(?1,?2,?3,?4,?5,'live',?6) ON CONFLICT(owner_id,path) DO UPDATE SET last_opened=excluded.last_opened,base_oid=excluded.base_oid",params![id,actor,name,root,base,Utc::now().to_rfc3339()])?;Ok(())})?;
        Ok(result)
    }
}
fn target(id: &Option<String>) -> Result<&str> {
    id.as_deref()
        .ok_or_else(|| Error::InvalidContract("target_id required".into()))
}
fn owned_draft(tx: &Transaction, actor: &str, id: &str) -> Result<(String, i64, String, String)> {
    tx.query_row(
        "SELECT mission_id,revision,status,project_id FROM drafts WHERE id=?1 AND owner_id=?2",
        params![id, actor],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )
    .optional()?
    .ok_or(Error::AuthRequired)
}
fn authorize_target(
    tx: &Transaction,
    actor: &str,
    target: Option<&str>,
    write_access: bool,
) -> Result<()> {
    if let Some(id) = target {
        let mission:Option<String>=tx.query_row("SELECT m.id FROM missions m WHERE m.owner_id=?2 AND (m.id=?1 OR EXISTS(SELECT 1 FROM drafts d WHERE d.id=?1 AND d.mission_id=m.id) OR EXISTS(SELECT 1 FROM contracts c WHERE c.task_id=?1 AND c.mission_id=m.id)) LIMIT 1",params![id,actor],|r|r.get(0)).optional()?;
        let mission = mission.ok_or(Error::AuthRequired)?;
        authorize_result(tx, actor, &json!({"mission_id":mission}))?;
        if write_access {
            let mut q=tx.prepare("SELECT repo_id FROM contracts WHERE mission_id=?1 UNION SELECT json_extract(plan_json,'$.task.repo_id') FROM drafts WHERE mission_id=?1 AND plan_json IS NOT NULL")?;
            let repos = q
                .query_map([&mission], |r| r.get::<_, String>(0))?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            for repo in repos {
                require_membership(tx, actor, &repo, true)?;
            }
        }
    }
    Ok(())
}
pub(crate) fn authorize_result(
    c: &rusqlite::Connection,
    actor: &str,
    result: &Value,
) -> Result<()> {
    if let Some(mission) = result["mission_id"].as_str() {
        let owned: bool = c.query_row(
            "SELECT EXISTS(SELECT 1 FROM missions WHERE id=?1 AND owner_id=?2)",
            params![mission, actor],
            |r| r.get(0),
        )?;
        if !owned {
            return Err(Error::AuthRequired);
        }
        let mut q=c.prepare("SELECT repo_id FROM contracts WHERE mission_id=?1 UNION SELECT json_extract(plan_json,'$.task.repo_id') FROM drafts WHERE mission_id=?1 AND plan_json IS NOT NULL")?;
        let repos = q
            .query_map([mission], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        for repo in repos {
            require_membership(c, actor, &repo, false)?;
        }
    }
    Ok(())
}
pub(crate) fn message(tx: &Transaction, draft: &str, role: &str, body: &str) -> Result<()> {
    tx.execute(
        "INSERT INTO conversation(draft_id,role,body,created_at) VALUES(?1,?2,?3,?4)",
        params![draft, role, body, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

#[cfg(test)]
mod replay_tests {
    use super::*;
    #[test]
    fn replayed_stop_cannot_signal_successor() {
        let dir = tempfile::tempdir().unwrap();
        let hub = Hub::open(dir.path()).unwrap();
        let run=hub.command_bytes("owner-demo",br#"{"schema_version":3,"command_id":"run","kind":"run","payload":{"goal":"x","project_id":"demo"}}"#).unwrap();
        let mission = run.result["mission_id"].as_str().unwrap();
        let body=serde_json::to_vec(&json!({"schema_version":3,"command_id":"stop","kind":"stop","target_id":mission,"expected_version":1,"payload":{}})).unwrap();
        let stop = hub.command_bytes("owner-demo", &body).unwrap();
        let successor = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        for old in stop.result["affected_job_ids"].as_array().unwrap() {
            hub.cancellations
                .lock()
                .unwrap()
                .insert(old.as_str().unwrap().into(), successor.clone());
        }
        hub.command_bytes("owner-demo", &body).unwrap();
        assert!(!successor.load(std::sync::atomic::Ordering::Acquire));
    }
}
