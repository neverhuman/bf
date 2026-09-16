use crate::digest::{parse_strict_json, require_utf8, sha256_hex};
use crate::domain::{cycle_in_ids, validate_profile, validate_task};
use crate::error::{Error, Result};
use crate::forge::{CreateOutcome, FakeForge};
use crate::gitutil::{commit_all, git, init_repo};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Transaction};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use uuid::Uuid;

const MIGRATION: &str = include_str!("../migrations/001_core.sql");
const TASK_JSON: &str = include_str!("../fixtures/protocol/task.json");
const GOOD_PY: &str = include_str!("../fixtures/gates/dedup/good.py");
const BUGGY_PY: &str = include_str!("../fixtures/gates/dedup/src/dedup.py");
const WRONG_PY: &str = include_str!("../fixtures/gates/dedup/wrong.py");

const PROFILE_JSON: &str = r#"{
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
    conn: Mutex<Connection>,
    forge: Mutex<FakeForge>,
    pub epoch: String,
    pub token: String,
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
    pub task_id: String,
    pub title: String,
    pub phase: String,
    pub why: String,
    pub pr: Option<Value>,
}

impl Hub {
    pub fn open(data_dir: &Path) -> Result<Self> {
        fs::create_dir_all(data_dir)?;
        fs::create_dir_all(data_dir.join("jobs"))?;
        fs::create_dir_all(data_dir.join("repos"))?;
        fs::create_dir_all(data_dir.join("artifacts"))?;
        let db_path = data_dir.join("hub.sqlite");
        let conn = Connection::open(&db_path)?;
        conn.busy_timeout(Duration::from_millis(5_000))?;
        conn.execute_batch(
            "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;",
        )?;
        let has = conn
            .query_row(
                "SELECT name FROM sqlite_master WHERE type='table' AND name='schema_meta'",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()?;
        if has.is_none() {
            conn.execute_batch(MIGRATION)?;
        }
        let hub = Self {
            data_dir: data_dir.to_path_buf(),
            conn: Mutex::new(conn),
            forge: Mutex::new(FakeForge::default()),
            epoch: format!("boot-{}", Uuid::new_v4()),
            token: Uuid::new_v4().to_string(),
        };
        hub.bootstrap()?;
        Ok(hub)
    }

    fn db(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("db lock")
    }

    fn bootstrap(&self) -> Result<()> {
        let conn = self.db();
        let n: i64 = conn.query_row("SELECT COUNT(*) FROM principals", [], |r| r.get(0))?;
        if n > 0 {
            return Ok(());
        }
        conn.execute_batch(
            r#"
            INSERT INTO principals(id,kind,active) VALUES
              ('owner-demo','human',1),
              ('agent-demo','agent',1),
              ('runner-fixture','runner',1),
              ('verifier-fixture','verifier',1),
              ('publisher-fixture','publisher',1);
            INSERT INTO repositories(id,canonical_forge,policy_json) VALUES
              ('repo-demo','fake','{"protected":true}');
            INSERT INTO memberships(principal_id,repo_id,role) VALUES
              ('owner-demo','repo-demo','administrator'),
              ('agent-demo','repo-demo','engineer'),
              ('runner-fixture','repo-demo','engineer'),
              ('verifier-fixture','repo-demo','engineer'),
              ('publisher-fixture','repo-demo','engineer');
            INSERT INTO domains(id,owner_id,policy_json) VALUES
              ('core','owner-demo','{}');
            INSERT INTO grants(id,issuer_id,subject_id,expires_at,revoked,invocation_allowance,used_invocations,capability_json)
              VALUES ('demo-grant','owner-demo','owner-demo','2099-01-01T00:00:00Z',0,10,0,'{"implement":true,"verify":true,"publish":true}');
            INSERT INTO budget_accounts(id,currency,limit_micros,actual_micros)
              VALUES ('demo-usd','USD',2000000,0);
            INSERT INTO runners(id,principal_id,incarnation_id,enabled,capacity)
              VALUES ('runner-fixture','runner-fixture','inc-1',1,2);
            "#,
        )?;
        let profile_bytes = PROFILE_JSON.as_bytes();
        validate_profile(&parse_strict_json(PROFILE_JSON)?)?;
        let digest = sha256_hex(profile_bytes);
        conn.execute(
            "INSERT INTO profiles(digest, raw_json) VALUES (?1,?2)",
            params![digest, profile_bytes],
        )?;
        Ok(())
    }

    pub fn sqlite_version(&self) -> Result<String> {
        Ok(self
            .db()
            .query_row("SELECT sqlite_version()", [], |r| r.get(0))?)
    }

    pub fn principal_kind(&self, id: &str) -> Result<String> {
        self.db()
            .query_row(
                "SELECT kind FROM principals WHERE id=?1 AND active=1",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| Error::AuthRequired)
    }

    pub fn lookup_session(&self, token: &str) -> Result<String> {
        self.db()
            .query_row(
                "SELECT principal_id FROM sessions WHERE token=?1",
                [token],
                |r| r.get(0),
            )
            .map_err(|_| Error::AuthRequired)
    }

    pub fn ensure_session(&self, principal_id: &str) -> Result<String> {
        let conn = self.db();
        if let Some(t) = conn
            .query_row(
                "SELECT token FROM sessions WHERE principal_id=?1 LIMIT 1",
                [principal_id],
                |r| r.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(t);
        }
        let token = self.token.clone();
        conn.execute(
            "INSERT INTO sessions(token,principal_id,created_at) VALUES (?1,?2,?3)",
            params![token, principal_id, Utc::now().to_rfc3339()],
        )?;
        Ok(token)
    }

    pub fn command_bytes(&self, actor_id: &str, raw: &[u8]) -> Result<Operation> {
        let text = require_utf8(raw)?;
        let value = parse_strict_json(text)?;
        crate::domain::require_schema_v3(&value)?;
        let command_id = crate::domain::require_id(&value, "command_id")?;
        let kind = crate::domain::require_id(&value, "kind")?;
        let digest = sha256_hex(raw);
        let mut conn = self.db();
        let tx = conn.transaction()?;
        if let Some((existing_digest, op_id)) = tx
            .query_row(
                "SELECT request_digest, operation_id FROM commands WHERE actor_id=?1 AND command_id=?2",
                params![actor_id, command_id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()?
        {
            if existing_digest != digest {
                return Err(Error::CommandConflict);
            }
            let result_json: String = tx.query_row(
                "SELECT result_json FROM operations WHERE id=?1",
                [&op_id],
                |r| r.get(0),
            )?;
            let result: Value = serde_json::from_str(&result_json)?;
            tx.commit()?;
            return Ok(Operation {
                id: op_id,
                status: "replayed".into(),
                result,
            });
        }
        let kind_actor: String = tx
            .query_row(
                "SELECT kind FROM principals WHERE id=?1 AND active=1",
                [actor_id],
                |r| r.get(0),
            )
            .map_err(|_| Error::AuthRequired)?;
        if matches!(
            kind.as_str(),
            "take" | "submit_human" | "grant_allowance" | "resolve_decision"
        ) && kind_actor != "human"
        {
            return Err(Error::PolicyDenied(
                "human-only command cannot be exercised by an agent".into(),
            ));
        }
        let payload = value.get("payload").cloned().unwrap_or(Value::Null);
        let op_id = format!("op-{}", Uuid::new_v4());
        let result = match kind.as_str() {
            "create_mission" => create_mission(&tx, actor_id, &payload)?,
            "run" => run_goal(&tx, actor_id, &payload)?,
            "take" => json!({"taken": true, "task_id": payload.get("task_id")}),
            "stop" | "pause" | "cancel" => {
                if let Some(id) = payload.get("mission_id").and_then(Value::as_str) {
                    tx.execute("UPDATE missions SET paused=1 WHERE id=?1", [id])?;
                }
                json!({"ok": true, "kind": kind})
            }
            other => {
                return Err(Error::InvalidContract(format!(
                    "unknown command kind {other}"
                )))
            }
        };
        tx.execute(
            "INSERT INTO commands(actor_id,command_id,request_digest,operation_id,payload_json) VALUES (?1,?2,?3,?4,?5)",
            params![actor_id, command_id, digest, op_id, text],
        )?;
        tx.execute(
            "INSERT INTO operations(id,actor_id,command_id,status,result_json) VALUES (?1,?2,?3,'accepted',?4)",
            params![op_id, actor_id, command_id, result.to_string()],
        )?;
        tx.execute(
            "INSERT INTO events(kind,producer_id,payload_json) VALUES ('command',?1,?2)",
            params![actor_id, result.to_string()],
        )?;
        tx.commit()?;
        Ok(Operation {
            id: op_id,
            status: "accepted".into(),
            result,
        })
    }

    pub fn work(&self) -> Result<Vec<WorkItem>> {
        let conn = self.db();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.phase, c.raw_json FROM tasks t JOIN contracts c ON c.task_id=t.id AND c.revision=t.active_revision",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Vec<u8>>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, phase, raw) = row?;
            let v: Value = serde_json::from_slice(&raw).unwrap_or(Value::Null);
            let title = v
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(id.as_str())
                .to_owned();
            let why = why_task(&conn, &id, &phase)?;
            let pr = latest_pr(&conn, &id)?;
            out.push(WorkItem {
                task_id: id,
                title,
                phase,
                why,
                pr,
            });
        }
        Ok(out)
    }

    pub fn doctor(&self) -> Value {
        let sqlite = self.sqlite_version().unwrap_or_else(|_| "unknown".into());
        let git_ok = std::process::Command::new("git")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let py_ok = std::process::Command::new("python3")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        let live = std::process::Command::new("codex")
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| o.status.success().then_some("codex"));
        json!({
            "ok": git_ok && py_ok,
            "sqlite": sqlite,
            "git": git_ok,
            "python3": py_ok,
            "live_executor": live,
            "note": if live.is_none() {
                "Fake executor only. Set a live grant to use a real coding provider."
            } else {
                "A live executor binary is on PATH; a grant is still required before paid work."
            }
        })
    }

    pub fn run_fixture(&self, name: &str) -> Result<DemoReceipt> {
        match name {
            "basic" => self.run_pipeline(false),
            "interrupted_publish" => self.run_pipeline(true),
            other => Err(Error::InvalidContract(format!("unknown fixture {other}"))),
        }
    }

    fn run_pipeline(&self, interrupted: bool) -> Result<DemoReceipt> {
        let task_val = parse_strict_json(TASK_JSON)?;
        validate_task(&task_val)?;
        let edges: Vec<(String, String)> = Vec::new();
        if cycle_in_ids(&edges) {
            return Err(Error::InvalidContract("cycle".into()));
        }

        let source = self.data_dir.join("repos/repo-demo");
        if source.exists() {
            fs::remove_dir_all(&source)?;
        }
        init_repo(&source)?;
        fs::create_dir_all(source.join("src"))?;
        fs::write(source.join("src/dedup.py"), BUGGY_PY)?;
        fs::write(source.join("README.md"), "demo fixture\n")?;
        let (base_oid, _) = commit_all(&source, "baseline buggy gate")?;

        git(&source, &["checkout", "-b", "bf/control"])?;
        fs::create_dir_all(source.join("tasks"))?;
        fs::write(source.join("tasks/T-001.json"), TASK_JSON)?;
        let (control_oid, _) = commit_all(&source, "activate T-001")?;
        git(&source, &["checkout", "main"])?;

        let profile_digest = sha256_hex(PROFILE_JSON.as_bytes());
        let contract_digest = sha256_hex(TASK_JSON.as_bytes());
        let mission_id = "M-001";
        let task_id = "T-001";

        {
            let exists: Option<String> = self
                .db()
                .query_row("SELECT id FROM missions WHERE id='M-001'", [], |r| r.get(0))
                .optional()?;
            if exists.is_some() {
                return self.current_demo_receipt(if interrupted {
                    "interrupted_publish"
                } else {
                    "basic"
                });
            }
        }
        {
            let mut conn = self.db();
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO missions(id,owner_id,domain_id,goal,version,paused,contract_json) VALUES (?1,'owner-demo','core','pr_ready',1,0,?2)",
                params![mission_id, TASK_JSON],
            )?;
            tx.execute(
                "INSERT INTO contracts(task_id,revision,mission_id,repo_id,digest,digest_scheme,git_oid,raw_json) VALUES (?1,1,?2,'repo-demo',?3,'sha256_exact_utf8_v3',?4,?5)",
                params![task_id, mission_id, contract_digest, control_oid, TASK_JSON.as_bytes()],
            )?;
            tx.execute(
                "INSERT INTO tasks(id,active_revision,version,phase,lineage_id) VALUES (?1,1,1,'ready','L-001')",
                [task_id],
            )?;
            tx.execute(
                "INSERT INTO completion_holds(id,task_id,amount_micros,remaining_micros,currency,state) VALUES ('CH-001',?1,500000,500000,'USD','held')",
                [task_id],
            )?;
            admit_implement_job(
                &tx,
                mission_id,
                task_id,
                &profile_digest,
                &self.epoch,
                200_000,
            )?;
            tx.execute(
                "INSERT INTO change_reservations(id,task_id,repo_id,resource_kind,resource_key,active,owner_id) VALUES ('res-1',?1,'repo-demo','path','src/dedup.py',1,'owner-demo')",
                [task_id],
            )?;
            tx.execute(
                "UPDATE tasks SET phase='working', writer_generation=1 WHERE id=?1",
                [task_id],
            )?;
            tx.commit()?;
        }

        let incarnation = Uuid::new_v4().to_string();
        let workspace = self.data_dir.join("jobs/J-impl").join(&incarnation);
        copy_dir(&source, &workspace)?;
        git(&workspace, &["checkout", "main"])?;

        {
            let conn = self.db();
            conn.execute(
                "UPDATE jobs SET lifecycle='running', occupancy='running' WHERE id='J-impl'",
                [],
            )?;
        }

        fs::write(workspace.join("src/dedup.py"), GOOD_PY)?;
        let (commit_oid, tree_oid) = commit_all(&workspace, "fix repeated delivery ids")?;
        let artifact_id = "ART-candidate";
        let artifact_bytes = GOOD_PY.as_bytes();
        let artifact_digest = sha256_hex(artifact_bytes);
        fs::write(
            self.data_dir.join("artifacts").join(artifact_id),
            artifact_bytes,
        )?;

        {
            let mut conn = self.db();
            let tx = conn.transaction()?;
            tx.execute(
                "INSERT INTO artifacts(id,repo_id,digest,byte_count,complete,metadata_json) VALUES (?1,'repo-demo',?2,?3,1,'{}')",
                params![artifact_id, artifact_digest, artifact_bytes.len() as i64],
            )?;
            tx.execute(
                "INSERT INTO candidates(id,task_id,task_revision,producer_job_id,commit_oid,tree_oid,base_oid,artifact_id,provenance_json) VALUES ('C-001',?1,1,'J-impl',?2,?3,?4,?5,'{\"fake\":true}')",
                params![task_id, commit_oid, tree_oid, base_oid, artifact_id],
            )?;
            tx.execute(
                "INSERT INTO selections(task_id,version,candidate_id) VALUES (?1,1,'C-001')",
                [task_id],
            )?;
            tx.execute(
                "UPDATE jobs SET lifecycle='succeeded', writer_authority='sealed', occupancy=?1 WHERE id='J-impl'",
                [if interrupted { "unconfirmed" } else { "stopped" }],
            )?;
            tx.execute("UPDATE tasks SET phase='checking' WHERE id=?1", [task_id])?;
            tx.commit()?;
        }

        if interrupted {
            let conn = self.db();
            conn.execute("UPDATE jobs SET occupancy='stopped' WHERE id='J-impl'", [])?;
        }

        admit_verify_and_run(self, mission_id, task_id, &workspace, &commit_oid)?;

        let logical_key = format!("pr:{task_id}:1");
        {
            let mut forge = self.forge.lock().expect("forge");
            forge.lose_next = interrupted;
            let outcome = forge.create(
                &logical_key,
                &commit_oid,
                &base_oid,
                "Reject repeated delivery IDs",
            );
            let mut conn = self.db();
            let tx = conn.transaction()?;
            match outcome {
                CreateOutcome::Confirmed(pr) => {
                    tx.execute(
                        "INSERT INTO effects(id,logical_key,request_digest,task_id,candidate_id,selection_version,grant_id,publisher_id,state,payload_json,receipt_json) VALUES ('E-pr',?1,?2,?3,'C-001',1,'demo-grant','publisher-fixture','confirmed',?4,?5)",
                        params![
                            logical_key,
                            sha256_hex(logical_key.as_bytes()),
                            task_id,
                            json!({"title": pr.title, "head": pr.head}).to_string(),
                            json!({"number": pr.number, "url": pr.url}).to_string()
                        ],
                    )?;
                    tx.execute(
                        "UPDATE tasks SET phase='review_ready' WHERE id=?1",
                        [task_id],
                    )?;
                }
                CreateOutcome::Lost => {
                    tx.execute(
                        "INSERT INTO effects(id,logical_key,request_digest,task_id,candidate_id,selection_version,grant_id,publisher_id,state,payload_json,receipt_json) VALUES ('E-pr',?1,?2,?3,'C-001',1,'demo-grant','publisher-fixture','outcome_unknown',?4,NULL)",
                        params![
                            logical_key,
                            sha256_hex(logical_key.as_bytes()),
                            task_id,
                            json!({"title": "Reject repeated delivery IDs", "head": commit_oid}).to_string()
                        ],
                    )?;
                }
            }
            tx.commit()?;
        }

        if interrupted {
            let pr = self
                .forge
                .lock()
                .expect("forge")
                .find(&logical_key)
                .cloned()
                .ok_or_else(|| {
                    Error::OutcomeUnknown("remote PR missing after lost reply".into())
                })?;
            let conn = self.db();
            conn.execute(
                "UPDATE effects SET state='confirmed', receipt_json=?1 WHERE id='E-pr'",
                params![
                    json!({"number": pr.number, "url": pr.url, "reconciled": true}).to_string()
                ],
            )?;
            conn.execute(
                "UPDATE tasks SET phase='review_ready' WHERE id=?1",
                [task_id],
            )?;
            let again = self.forge.lock().expect("forge").create(
                &logical_key,
                &commit_oid,
                &base_oid,
                "Reject repeated delivery IDs",
            );
            match again {
                CreateOutcome::Confirmed(second) if second.number != pr.number => {
                    return Err(Error::Other("duplicate PR created after reconcile".into()));
                }
                _ => {}
            }
        }

        let conn = self.db();
        let phase: String =
            conn.query_row("SELECT phase FROM tasks WHERE id=?1", [task_id], |r| {
                r.get(0)
            })?;
        let author_authority: String = conn.query_row(
            "SELECT writer_authority FROM jobs WHERE id='J-impl'",
            [],
            |r| r.get(0),
        )?;
        let occupancy: String =
            conn.query_row("SELECT occupancy FROM jobs WHERE id='J-impl'", [], |r| {
                r.get(0)
            })?;
        let effect_state: String =
            conn.query_row("SELECT state FROM effects WHERE id='E-pr'", [], |r| {
                r.get(0)
            })?;
        let check_result: String = conn.query_row(
            "SELECT result FROM evidence WHERE check_id='dedup-repeat'",
            [],
            |r| r.get(0),
        )?;
        let receipt_json: Option<String> = conn
            .query_row(
                "SELECT receipt_json FROM effects WHERE id='E-pr'",
                [],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        let pr_val: Option<Value> = receipt_json.and_then(|s| serde_json::from_str(&s).ok());
        drop(conn);

        Ok(DemoReceipt {
            fixture: if interrupted {
                "interrupted_publish".into()
            } else {
                "basic".into()
            },
            mission_id: mission_id.into(),
            task_id: task_id.into(),
            candidate_id: "C-001".into(),
            check_result,
            pr_number: pr_val
                .as_ref()
                .and_then(|v| v.get("number").and_then(Value::as_u64)),
            pr_url: pr_val
                .as_ref()
                .and_then(|v| v.get("url").and_then(Value::as_str))
                .map(str::to_owned),
            effect_state,
            author_authority,
            occupancy,
            phase,
            fake: true,
            limitations: vec![
                "Fake executor and fake forge.".into(),
                "Not a live-provider certification.".into(),
            ],
        })
    }

    pub fn gate_at032(&self, work: &Path) -> Result<(bool, bool, bool)> {
        let check = crate_root().join("fixtures/gates/dedup/check.py");
        fs::create_dir_all(work.join("src"))?;
        fs::write(work.join("src/dedup.py"), BUGGY_PY)?;
        let buggy_fail = run_check(&check, work)? != 0;
        fs::write(work.join("src/dedup.py"), GOOD_PY)?;
        let good_pass = run_check(&check, work)? == 0;
        fs::write(work.join("src/dedup.py"), WRONG_PY)?;
        let wrong_fail = run_check(&check, work)? != 0;
        Ok((buggy_fail, good_pass, wrong_fail))
    }

    pub fn occupied_slots(&self) -> Result<i64> {
        Ok(self.db().query_row(
            "SELECT COUNT(*) FROM jobs WHERE occupancy IN ('allocated','running','unconfirmed')",
            [],
            |r| r.get(0),
        )?)
    }

    pub fn try_second_writer(&self) -> Result<()> {
        let conn = self.db();
        let profile_digest = sha256_hex(PROFILE_JSON.as_bytes());
        let envelope = json!({"purpose": "implement", "dup": true});
        let err = conn.execute(
            "INSERT INTO jobs(id,mission_id,task_id,task_revision,purpose,lane,executor_kind,profile_digest,runner_id,assignment_generation,writer_generation,authority_epoch,grant_id,deadline_at,lifecycle,writer_authority,occupancy,input_digest,envelope_json)
             VALUES ('J-impl-dup','M-001','T-001',1,'implement','shipping','model',?1,'runner-fixture',2,1,?2,'demo-grant','2099-01-01T00:00:00Z','prepared','active','allocated',?3,?4)",
            params![
                profile_digest,
                self.epoch,
                sha256_hex(envelope.to_string().as_bytes()),
                envelope.to_string()
            ],
        );
        match err {
            Err(_) => Ok(()),
            Ok(_) => Err(Error::Other("second writer was admitted".into())),
        }
    }

    fn current_demo_receipt(&self, fixture: &str) -> Result<DemoReceipt> {
        let conn = self.db();
        let phase: String =
            conn.query_row("SELECT phase FROM tasks WHERE id='T-001'", [], |r| r.get(0))?;
        let author_authority: String = conn.query_row(
            "SELECT writer_authority FROM jobs WHERE id='J-impl'",
            [],
            |r| r.get(0),
        )?;
        let occupancy: String =
            conn.query_row("SELECT occupancy FROM jobs WHERE id='J-impl'", [], |r| {
                r.get(0)
            })?;
        let effect_state: String =
            conn.query_row("SELECT state FROM effects WHERE id='E-pr'", [], |r| {
                r.get(0)
            })?;
        let check_result: String = conn.query_row(
            "SELECT result FROM evidence WHERE check_id='dedup-repeat'",
            [],
            |r| r.get(0),
        )?;
        let receipt_json: Option<String> = conn
            .query_row(
                "SELECT receipt_json FROM effects WHERE id='E-pr'",
                [],
                |r| r.get::<_, Option<String>>(0),
            )
            .optional()?
            .flatten();
        let pr_val: Option<Value> = receipt_json.and_then(|s| serde_json::from_str(&s).ok());
        Ok(DemoReceipt {
            fixture: fixture.into(),
            mission_id: "M-001".into(),
            task_id: "T-001".into(),
            candidate_id: "C-001".into(),
            check_result,
            pr_number: pr_val
                .as_ref()
                .and_then(|v| v.get("number").and_then(Value::as_u64)),
            pr_url: pr_val
                .as_ref()
                .and_then(|v| v.get("url").and_then(Value::as_str))
                .map(str::to_owned),
            effect_state,
            author_authority,
            occupancy,
            phase,
            fake: true,
            limitations: vec!["Replayed from existing hub state.".into()],
        })
    }

    pub fn writer_cannot_spend_completion(&self) -> Result<()> {
        let conn = self.db();
        let remaining: i64 = conn.query_row(
            "SELECT remaining_micros FROM completion_holds WHERE id='CH-001'",
            [],
            |r| r.get(0),
        )?;
        let impl_uses_completion: i64 = conn.query_row(
            "SELECT COUNT(*) FROM budget_holds WHERE job_id='J-impl' AND completion_hold_id IS NOT NULL",
            [],
            |r| r.get(0),
        )?;
        if impl_uses_completion != 0 || !(400_000..=500_000).contains(&remaining) {
            return Err(Error::BudgetUnavailable(
                "writer consumed the completion reserve".into(),
            ));
        }
        Ok(())
    }

    pub fn parse_result_frame(bytes: &[u8]) -> Result<Value> {
        let text = require_utf8(bytes)?;
        if !text.trim_end().ends_with('}') {
            return Err(Error::InvalidContract("truncated result frame".into()));
        }
        parse_strict_json(text)
    }

    pub fn set_grant_revoked(&self, revoked: bool) -> Result<()> {
        self.db().execute(
            "UPDATE grants SET revoked=?1 WHERE id='demo-grant'",
            [if revoked { 1 } else { 0 }],
        )?;
        Ok(())
    }
}

fn create_mission(tx: &Transaction, actor_id: &str, payload: &Value) -> Result<Value> {
    let id = payload
        .get("id")
        .and_then(Value::as_str)
        .map(|s| s.to_owned())
        .unwrap_or_else(|| format!("M-{}", Uuid::new_v4()));
    let goal = payload
        .get("goal")
        .and_then(Value::as_str)
        .unwrap_or("pr_ready");
    let owner = payload
        .get("owner_id")
        .and_then(Value::as_str)
        .unwrap_or(actor_id);
    if owner != actor_id {
        let kind: String =
            tx.query_row("SELECT kind FROM principals WHERE id=?1", [actor_id], |r| {
                r.get(0)
            })?;
        if kind != "human" {
            return Err(Error::PolicyDenied(
                "body owner fields never authenticate".into(),
            ));
        }
    }
    tx.execute(
        "INSERT INTO missions(id,owner_id,domain_id,goal,version,paused,contract_json) VALUES (?1,?2,'core',?3,1,0,?4)",
        params![id, actor_id, goal, payload.to_string()],
    )?;
    Ok(json!({"mission_id": id, "goal": goal}))
}

fn run_goal(tx: &Transaction, actor_id: &str, payload: &Value) -> Result<Value> {
    let goal = payload.get("goal").and_then(Value::as_str).unwrap_or("");
    create_mission(tx, actor_id, &json!({"goal": "pr_ready", "title": goal}))
}

fn admit_implement_job(
    tx: &Transaction,
    mission_id: &str,
    task_id: &str,
    profile_digest: &str,
    epoch: &str,
    amount: i64,
) -> Result<()> {
    let occupied: i64 = tx.query_row(
        "SELECT COUNT(*) FROM jobs WHERE occupancy IN ('allocated','running','unconfirmed')",
        [],
        |r| r.get(0),
    )?;
    let cap: i64 = tx.query_row(
        "SELECT capacity FROM runners WHERE id='runner-fixture'",
        [],
        |r| r.get(0),
    )?;
    if occupied >= cap {
        return Err(Error::ResourceConflict("runner capacity".into()));
    }
    let remaining: i64 = tx.query_row(
        "SELECT limit_micros - actual_micros FROM budget_accounts WHERE id='demo-usd'",
        [],
        |r| r.get(0),
    )?;
    if remaining < amount + 500_000 {
        return Err(Error::BudgetUnavailable(
            "need writer amount plus completion reserve".into(),
        ));
    }
    let active: i64 = tx.query_row(
        "SELECT COUNT(*) FROM jobs WHERE task_id=?1 AND writer_authority='active'",
        [task_id],
        |r| r.get(0),
    )?;
    if active > 0 {
        return Err(Error::ResourceConflict(
            "one current shipping writer".into(),
        ));
    }
    let gen: i64 = tx.query_row(
        "SELECT writer_generation FROM tasks WHERE id=?1",
        [task_id],
        |r| r.get(0),
    )?;
    let writer_generation = gen + 1;
    let envelope = json!({"purpose": "implement", "task_id": task_id});
    tx.execute(
        "INSERT INTO jobs(id,mission_id,task_id,task_revision,purpose,lane,executor_kind,profile_digest,runner_id,assignment_generation,writer_generation,authority_epoch,grant_id,deadline_at,lifecycle,writer_authority,occupancy,input_digest,envelope_json)
         VALUES ('J-impl',?1,?2,1,'implement','shipping','model',?3,'runner-fixture',1,?4,?5,'demo-grant',?6,'prepared','active','allocated',?7,?8)",
        params![
            mission_id,
            task_id,
            profile_digest,
            writer_generation,
            epoch,
            "2099-01-01T00:00:00Z",
            sha256_hex(envelope.to_string().as_bytes()),
            envelope.to_string()
        ],
    )?;
    tx.execute(
        "INSERT INTO budget_holds(id,job_id,account_id,completion_hold_id,amount_micros,state) VALUES ('BH-impl','J-impl','demo-usd',NULL,?1,'held')",
        [amount],
    )?;
    tx.execute(
        "UPDATE budget_accounts SET actual_micros = actual_micros + ?1 WHERE id='demo-usd'",
        [amount],
    )?;
    Ok(())
}

fn admit_verify_and_run(
    hub: &Hub,
    mission_id: &str,
    task_id: &str,
    workspace: &Path,
    subject: &str,
) -> Result<()> {
    {
        let mut conn = hub.db();
        let tx = conn.transaction()?;
        let envelope = json!({"purpose": "verify", "check_id": "dedup-repeat"});
        tx.execute(
            "INSERT INTO jobs(id,mission_id,task_id,task_revision,purpose,lane,executor_kind,profile_digest,runner_id,assignment_generation,writer_generation,authority_epoch,grant_id,deadline_at,lifecycle,writer_authority,occupancy,input_digest,envelope_json)
             VALUES ('J-verify',?1,?2,1,'verify','shipping','check',NULL,'runner-fixture',1,NULL,?3,'demo-grant','2099-01-01T00:00:00Z','running','none','running',?4,?5)",
            params![
                mission_id,
                task_id,
                hub.epoch,
                sha256_hex(subject.as_bytes()),
                envelope.to_string()
            ],
        )?;
        tx.execute(
            "INSERT INTO budget_holds(id,job_id,account_id,completion_hold_id,amount_micros,state) VALUES ('BH-verify','J-verify','demo-usd','CH-001',100000,'held')",
            [],
        )?;
        tx.execute(
            "UPDATE completion_holds SET remaining_micros = remaining_micros - 100000, state='assigned' WHERE id='CH-001'",
            [],
        )?;
        tx.commit()?;
    }
    let check = crate_root().join("fixtures/gates/dedup/check.py");
    let code = run_check(&check, workspace)?;
    let result = if code == 0 { "pass" } else { "fail" };
    let receipt = json!({
        "schema_version": 3,
        "id": "EV-001",
        "stage": "candidate",
        "mission_id": mission_id,
        "task_id": task_id,
        "job_id": "J-verify",
        "subject_digest": sha256_hex(subject.as_bytes()),
        "check_id": "dedup-repeat",
        "producer_id": "verifier-fixture",
        "result": result,
        "suites": [{"name": "protected_dedup", "executed": 2, "failed": if code == 0 {0} else {1}, "skipped": 0}]
    });
    let mut conn = hub.db();
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO evidence(id,job_id,candidate_id,subject_digest,producer_id,check_id,result,receipt_json) VALUES ('EV-001','J-verify','C-001',?1,'verifier-fixture','dedup-repeat',?2,?3)",
        params![sha256_hex(subject.as_bytes()), result, receipt.to_string()],
    )?;
    tx.execute(
        "UPDATE jobs SET lifecycle='succeeded', occupancy='stopped' WHERE id='J-verify'",
        [],
    )?;
    if result != "pass" {
        tx.execute("UPDATE tasks SET phase='failed' WHERE id=?1", [task_id])?;
        tx.commit()?;
        return Err(Error::Other("independent check failed".into()));
    }
    tx.commit()?;
    Ok(())
}

fn run_check(check: &Path, workspace: &Path) -> Result<i32> {
    let out = std::process::Command::new("python3")
        .arg(check)
        .arg(workspace)
        .output()
        .map_err(|e| Error::Other(format!("python3: {e}")))?;
    Ok(out.status.code().unwrap_or(2))
}

fn copy_dir(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn why_task(conn: &Connection, task_id: &str, phase: &str) -> Result<String> {
    let check: Option<String> = conn
        .query_row(
            "SELECT result FROM evidence WHERE check_id='dedup-repeat' ORDER BY rowid DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .optional()?;
    let effect: Option<String> = conn
        .query_row(
            "SELECT state FROM effects WHERE task_id=?1 ORDER BY rowid DESC LIMIT 1",
            [task_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(format!(
        "phase={phase}; check={}; forge={}",
        check.unwrap_or_else(|| "none".into()),
        effect.unwrap_or_else(|| "none".into())
    ))
}

fn latest_pr(conn: &Connection, task_id: &str) -> Result<Option<Value>> {
    let receipt: Option<String> = conn
        .query_row(
            "SELECT receipt_json FROM effects WHERE task_id=?1 AND state='confirmed'",
            [task_id],
            |r| r.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten();
    Ok(receipt.and_then(|s| serde_json::from_str(&s).ok()))
}
