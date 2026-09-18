use crate::hub::{DemoReceipt, Hub, BUGGY_PY, GOOD_PY, WRONG_PY};
use crate::{Error, Result};
use rusqlite::params;
use serde_json::{json, Value};
use std::path::Path;

impl Hub {
    pub fn run_fixture(&self, name: &str) -> Result<DemoReceipt> {
        let _guard = self
            .fixture_lock
            .try_lock()
            .map_err(|_| Error::ResourceConflict("fixture already running".into()))?;
        if !matches!(name, "basic" | "interrupted_publish") {
            return Err(Error::InvalidContract("unknown fixture".into()));
        }
        let run = json!({"schema_version":3,"command_id":"demo-plan","kind":"run","target_id":null,"expected_version":null,"payload":{"project_id":"demo","goal":"Reject repeated delivery IDs"}});
        let operation = self.command_bytes("owner-demo", &serde_json::to_vec(&run)?)?;
        self.drive()?;
        let draft = operation.result["draft_id"].as_str().unwrap();
        let start = json!({"schema_version":3,"command_id":"demo-start","kind":"start_work","target_id":draft,"expected_version":1,"payload":{}});
        let op = self.command_bytes("owner-demo", &serde_json::to_vec(&start)?)?;
        if name == "interrupted_publish" {
            self.set_fixture_fault("after_remote")?;
        }
        if let Err(error) = self.drive() {
            if error.code() != "OUTCOME_UNKNOWN" {
                return Err(error);
            }
            self.reconcile_effects()?;
        }
        self.receipt(name, op.result["task_id"].as_str().unwrap())
    }
    pub fn receipt(&self, name: &str, task: &str) -> Result<DemoReceipt> {
        let name = name.to_owned();
        let task = task.to_owned();
        self.db.call(move|c|{
        let (mission,phase):(String,String)=c.query_row("SELECT c.mission_id,t.phase FROM tasks t JOIN contracts c ON c.task_id=t.id AND c.revision=t.active_revision WHERE t.id=?1",[&task],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let (candidate,check):(String,String)=c.query_row("SELECT s.candidate_id,e.result FROM selections s JOIN evidence e ON e.candidate_id=s.candidate_id WHERE s.task_id=?1 ORDER BY e.rowid DESC LIMIT 1",[&task],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let (authority,occupancy):(String,String)=c.query_row("SELECT j.writer_authority,j.occupancy FROM jobs j JOIN candidates ca ON ca.producer_job_id=j.id WHERE ca.id=?1",[&candidate],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let (state,receipt):(String,Option<String>)=c.query_row("SELECT state,receipt_json FROM effects WHERE task_id=?1",[&task],|r|Ok((r.get(0)?,r.get(1)?)))?;
        let receipt:Value=receipt.and_then(|s|serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
        Ok(DemoReceipt{fixture:name,mission_id:mission,task_id:task,candidate_id:candidate,check_result:check,pr_number:receipt["number"].as_u64(),pr_url:receipt["url"].as_str().map(str::to_owned),effect_state:state,author_authority:authority,occupancy,phase,fake:true,limitations:vec!["Deterministic fixture corpus only; no live provider, containment or independent live review qualification.".into()]})
    })
    }
    pub fn gate_at032(&self, work: &Path) -> Result<(bool, bool, bool)> {
        std::fs::create_dir_all(work.join("src"))?;
        let mut results = Vec::new();
        for source in [BUGGY_PY, GOOD_PY, WRONG_PY] {
            std::fs::write(work.join("src/dedup.py"), source)?;
            results.push(crate::verification::verify_fixture(work)?["result"] == "pass");
        }
        Ok((!results[0], results[1], !results[2]))
    }
    pub fn verify_fixture_source(&self, work: &Path) -> Result<Value> {
        crate::verification::verify_fixture(work)
    }
    pub fn writer_cannot_spend_completion(&self) -> Result<()> {
        self.db.call(|c|{
        let n:i64=c.query_row("SELECT count(*) FROM budget_holds h JOIN jobs j ON j.id=h.job_id WHERE j.purpose='implement' AND h.completion_hold_id IS NOT NULL",[],|r|r.get(0))?;
        if n!=0{return Err(Error::BudgetUnavailable("writer consumed verification reserve".into()))}Ok(())
    })
    }
    pub fn try_second_writer(&self) -> Result<()> {
        self.db.call(|c|{
        let tx=c.transaction()?;
        let task:String=tx.query_row("SELECT id FROM tasks LIMIT 1",[],|r|r.get(0))?;
        // Independent of generation uniqueness: make one writer active, then attempt a distinct generation.
        tx.execute("UPDATE jobs SET writer_authority='active' WHERE task_id=?1 AND purpose='implement'",[&task])?;
        let result=tx.execute("INSERT INTO jobs SELECT 'second-writer',mission_id,task_id,task_revision,purpose,lane,executor_kind,profile_digest,runner_id,assignment_generation+1,writer_generation+1,authority_epoch,grant_id,deadline_at,'prepared','active','allocated',input_digest,envelope_json FROM jobs WHERE task_id=?1 AND purpose='implement' LIMIT 1",params![task]);
        if result.is_ok(){return Err(Error::ResourceConflict("second writer admitted".into()))}Ok(())
    })
    }
}
