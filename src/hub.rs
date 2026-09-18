use crate::digest::sha256_hex;
use crate::storage::Database;
use crate::{Error, Result};
use chrono::{Duration, Utc};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use uuid::Uuid;

pub(crate) const TASK_JSON: &str = include_str!("../fixtures/protocol/task.json");
pub(crate) const GOOD_PY: &str = include_str!("../fixtures/gates/dedup/good.py");
pub(crate) const BUGGY_PY: &str = include_str!("../fixtures/gates/dedup/src/dedup.py");
pub(crate) const WRONG_PY: &str = include_str!("../fixtures/gates/dedup/wrong.py");
pub(crate) const PROFILE_JSON: &str = r#"{
  "schema_version": 3,
  "id": "profile-fake",
  "revision": 1,
  "harness": {
    "name": "fake",
    "source_revision": "fixture-v1",
    "binary_digest": "1111111111111111111111111111111111111111111111111111111111111111",
    "adapter_revision": "fake-v1",
    "transport": "jsonl"
  },
  "provider_id": "fake",
  "model_id": "deterministic-patch",
  "settings": {},
  "context_strategy": "compact-v1",
  "prompt_digest": "2222222222222222222222222222222222222222222222222222222222222222",
  "skills_digest": "3333333333333333333333333333333333333333333333333333333333333333",
  "initial_memory_digest": "4444444444444444444444444444444444444444444444444444444444444444",
  "engine_ref": "engine-fake",
  "environment_digest": "5555555555555555555555555555555555555555555555555555555555555555",
  "hardware_class": "fixture",
  "delegation": {
    "mode": "disabled",
    "max_depth": 0,
    "max_children": 0,
    "usage_basis": "inclusive_root"
  },
  "refinement": "frozen"
}"#;

pub fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
pub struct Hub {
    pub data_dir: PathBuf,
    pub(crate) db: Database,
    pub epoch: String,
    pub(crate) driving: AtomicBool,
    pub(crate) fixture_lock: Mutex<()>,
    pub(crate) cancellations: Mutex<std::collections::HashMap<String, std::sync::Arc<AtomicBool>>>,
    _instance: File,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: String,
    pub status: String,
    pub result: Value,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoReceipt {
    pub fixture: String,
    pub mission_id: String,
    pub task_id: String,
    pub candidate_id: String,
    pub check_result: String,
    pub pr_number: Option<u64>,
    pub pr_url: Option<String>,
    pub effect_state: String,
    pub author_authority: String,
    pub occupancy: String,
    pub phase: String,
    pub fake: bool,
    pub limitations: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkItem {
    pub cursor: i64,
    pub task_id: String,
    pub mission_id: String,
    pub version: i64,
    pub title: String,
    pub phase: String,
    pub why: String,
    pub pr: Option<Value>,
}
impl Hub {
    pub fn open(data_dir: &Path) -> Result<Self> {
        if data_dir
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            return Err(Error::PolicyDenied(
                "hub directory cannot be a symlink".into(),
            ));
        }
        fs::create_dir_all(data_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(data_dir, fs::Permissions::from_mode(0o700))?;
        }
        let instance = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(data_dir.join("hub.lock"))?;
        instance.try_lock().map_err(|_| {
            Error::ResourceConflict("hub already running; reconnect using endpoint.json".into())
        })?;
        for name in ["jobs", "repos", "artifacts"] {
            fs::create_dir_all(data_dir.join(name))?;
        }
        let hub = Self {
            data_dir: fs::canonicalize(data_dir)?,
            db: Database::open(&data_dir.join("hub.sqlite"))?,
            epoch: format!("boot-{}", Uuid::new_v4()),
            driving: AtomicBool::new(false),
            fixture_lock: Mutex::new(()),
            cancellations: Mutex::new(std::collections::HashMap::new()),
            _instance: instance,
        };
        hub.bootstrap()?;
        hub.recover()?;
        Ok(hub)
    }
    fn bootstrap(&self) -> Result<()> {
        self.db.call(|conn| {
            let tx=conn.transaction()?;
            let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM principals)",[],|r|r.get(0))?;
            if exists{return Ok(())}
            tx.execute_batch(r#"
                INSERT OR IGNORE INTO principals VALUES('owner-demo','human',1),('agent-demo','agent',1),('runner-fixture','runner',1),('verifier-fixture','verifier',1),('publisher-fixture','publisher',1);
                INSERT OR IGNORE INTO repositories VALUES('repo-demo','fake','{"fixture_only":true}');
                INSERT OR IGNORE INTO memberships VALUES('owner-demo','repo-demo','administrator'),('agent-demo','repo-demo','engineer');
                INSERT OR IGNORE INTO domains VALUES('core','owner-demo','{}');
                INSERT OR IGNORE INTO grants(id,issuer_id,subject_id,expires_at,revoked,invocation_allowance,used_invocations,capability_json,fixture_only)
                VALUES('demo-grant','owner-demo','owner-demo','2099-01-01T00:00:00Z',0,100,0,'{"implement":true,"plan":true,"verify":true,"publish":true}',1);
                INSERT OR IGNORE INTO budget_accounts VALUES('demo-usd','USD',2000000,0);
                INSERT OR IGNORE INTO runners VALUES('runner-fixture','runner-fixture','fixture-process',1,4);
                INSERT OR IGNORE INTO projects VALUES('demo','owner-demo','Delivery ID demo','fixture://dedup','fixture-v1','fixture','2026-01-01T00:00:00Z');
            "#)?;
            tx.execute("INSERT OR IGNORE INTO profiles VALUES(?1,?2)",params![sha256_hex(PROFILE_JSON.as_bytes()),PROFILE_JSON.as_bytes()])?;
            tx.commit()?;
            Ok(())
        })
    }
    pub fn sqlite_version(&self) -> Result<String> {
        self.db
            .call(|c| Ok(c.query_row("SELECT sqlite_version()", [], |r| r.get(0))?))
    }
    pub fn principal_kind(&self, id: &str) -> Result<String> {
        let id = id.to_owned();
        self.db.call(move |c| active_actor(c, &id))
    }
    pub fn ensure_session(&self, principal: &str) -> Result<String> {
        let principal = principal.to_owned();
        let token = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        let digest = sha256_hex(token.as_bytes());
        self.db.call(move |c| {
            active_actor(c,&principal)?;
            c.execute("INSERT INTO sessions(token,principal_id,created_at,expires_at) VALUES(?1,?2,?3,?4)",params![digest,principal,Utc::now().to_rfc3339(),(Utc::now()+Duration::hours(8)).to_rfc3339()])?;
            Ok(())
        })?;
        Ok(token)
    }
    pub fn lookup_session(&self, token: &str) -> Result<String> {
        let hash = sha256_hex(token.as_bytes());
        self.db.call(move |c| c.query_row("SELECT s.principal_id FROM sessions s JOIN principals p ON p.id=s.principal_id WHERE s.token=?1 AND s.revoked=0 AND p.active=1 AND julianday(s.expires_at)>julianday('now')",[hash],|r|r.get(0)).map_err(|_|Error::AuthRequired))
    }
    pub fn revoke_session(&self, token: &str) -> Result<()> {
        let hash = sha256_hex(token.as_bytes());
        self.db.call(move |c| {
            c.execute("UPDATE sessions SET revoked=1 WHERE token=?1", [hash])?;
            Ok(())
        })
    }
    pub fn change_cursor(&self, actor: &str) -> Result<i64> {
        let actor = actor.to_owned();
        self.db.call(move|c|{
        active_actor(c,&actor)?;require_membership(c,&actor,"repo-demo",false)?;
        Ok(c.query_row("SELECT COALESCE(max(e.seq),0) FROM events e JOIN memberships access ON access.repo_id=e.scope_repo_id AND access.principal_id=?1 LEFT JOIN jobs j ON j.id=e.job_id LEFT JOIN missions m ON m.id=j.mission_id WHERE e.producer_id=?1 OR m.owner_id=?1",[actor],|r|r.get(0))?)
    })
    }
    pub fn doctor(&self) -> Value {
        json!({"ok":true,"sqlite":self.sqlite_version().ok(),"live_executor":null,
            "provider":self.account_status().unwrap_or(Value::Null),
            "live_blockers":["Operating HOLD","No live grant","Disposable Linux runner not qualified","Codex transport not qualified"],
            "note":"Fixture execution only. Account installation, authentication and qualification are separate requirements."})
    }
    pub fn work(&self) -> Result<Vec<WorkItem>> {
        self.work_for("owner-demo", None, 50)
    }
    pub fn work_for(
        &self,
        actor: &str,
        before: Option<i64>,
        limit: usize,
    ) -> Result<Vec<WorkItem>> {
        let actor = actor.to_owned();
        self.db.call(move |c| {
            active_actor(c,&actor)?;
            let mut q=c.prepare("SELECT p.seq,p.task_id,p.mission_id,p.version,p.title,p.phase,p.why,p.pr_json FROM task_projection p JOIN memberships m ON m.repo_id=p.repo_id AND m.principal_id=?1 WHERE p.owner_id=?1 AND p.seq<?2 ORDER BY p.seq DESC LIMIT ?3")?;
            let rows=q.query_map(params![actor,before.unwrap_or(i64::MAX),limit.clamp(1,100) as i64],|r| {
                let pr:Option<String>=r.get(7)?;
                Ok(WorkItem{cursor:r.get(0)?,task_id:r.get(1)?,mission_id:r.get(2)?,version:r.get(3)?,title:r.get(4)?,phase:r.get(5)?,why:r.get(6)?,pr:pr.and_then(|s|serde_json::from_str(&s).ok())})
            })?;
            Ok(rows.collect::<std::result::Result<Vec<_>,_>>()?)
        })
    }
    pub fn operation(&self, actor: &str, id: &str) -> Result<Operation> {
        let actor = actor.to_owned();
        let id = id.to_owned();
        self.db.call(move |c| {
            active_actor(c, &actor)?;
            require_membership(c, &actor, "repo-demo", false)?;
            let op = c
                .query_row(
                    "SELECT id,status,result_json FROM operations WHERE id=?1 AND actor_id=?2",
                    params![id, actor],
                    |r| {
                        let raw: String = r.get(2)?;
                        Ok(Operation {
                            id: r.get(0)?,
                            status: r.get(1)?,
                            result: serde_json::from_str(&raw).unwrap_or(Value::Null),
                        })
                    },
                )
                .optional()?
                .ok_or(Error::AuthRequired)?;
            crate::commands::authorize_result(c, &actor, &op.result)?;
            Ok(op)
        })
    }
    pub fn set_grant_revoked(&self, revoked: bool) -> Result<()> {
        self.db.call(move |c| {
            c.execute(
                "UPDATE grants SET revoked=?1 WHERE id='demo-grant'",
                [revoked],
            )?;
            Ok(())
        })
    }
    pub fn occupied_slots(&self) -> Result<i64> {
        self.db.call(|c| {
            Ok(c.query_row(
                "SELECT COUNT(*) FROM jobs WHERE occupancy<>'stopped'",
                [],
                |r| r.get(0),
            )?)
        })
    }
    pub fn parse_result_frame(bytes: &[u8]) -> Result<Value> {
        crate::digest::parse_strict_json(crate::digest::require_utf8(bytes)?)
    }
    pub(crate) fn claim_driver(&self) -> bool {
        !self.driving.swap(true, Ordering::AcqRel)
    }
    pub(crate) fn release_driver(&self) {
        self.driving.store(false, Ordering::Release);
    }
}
impl Drop for Hub {
    fn drop(&mut self) {
        self.db.shutdown();
        let _ = self._instance.unlock();
    }
}
pub(crate) fn active_actor(c: &rusqlite::Connection, id: &str) -> Result<String> {
    c.query_row(
        "SELECT kind FROM principals WHERE id=?1 AND active=1",
        [id],
        |r| r.get(0),
    )
    .map_err(|_| Error::AuthRequired)
}
pub(crate) fn require_membership(
    c: &rusqlite::Connection,
    actor: &str,
    repo: &str,
    write: bool,
) -> Result<()> {
    let role: Option<String> = c
        .query_row(
            "SELECT role FROM memberships WHERE principal_id=?1 AND repo_id=?2",
            params![actor, repo],
            |r| r.get(0),
        )
        .optional()?;
    match role.as_deref() {
        Some("administrator" | "engineer") => Ok(()),
        Some("observer") if !write => Ok(()),
        _ => Err(Error::PolicyDenied(
            "current repository access required".into(),
        )),
    }
}
