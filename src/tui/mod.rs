//! The operator screen. Bare `bf` opens it. This module is filled in by plan PR 6 (screens) and
//! PR 7 (wiring); until then `run` prints the plain snapshot and reports what is missing.
use crate::{Error, Result};
use serde::Serialize;

/// What the screen reads and the few things it may do. Implemented over agents + board + prs in PR 7.
pub trait Source: Send + 'static {
    fn snapshot(&mut self) -> Snapshot;
    fn stop(&mut self, pid: i64) -> std::result::Result<String, String>;
    fn note(&mut self, to_agent: Option<String>, body: String) -> std::result::Result<(), String>;
    fn expire_claim(&mut self, claim_id: &str) -> std::result::Result<(), String>;
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Snapshot {
    pub at: String,
    pub agents: Vec<AgentRow>,
    pub claims: Vec<ClaimRow>,
    pub recent: Vec<EntryRow>,
    pub prs: Vec<PrRow>,
    pub prs_stale_since: Option<String>,
    pub load: Option<f64>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentRow {
    pub provider: String,
    pub pid: i64,
    pub state: String,
    pub waiting_for: Option<String>,
    pub age_secs: Option<u64>,
    pub cwd: String,
    pub branch: Option<String>,
    pub title: Option<String>,
    pub last_prompt: Option<String>,
    pub detail: serde_json::Value,
    pub transcript_tail: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct ClaimRow {
    pub id: String,
    pub agent: String,
    pub provider: String,
    pub repo: String,
    pub paths: Vec<String>,
    pub expires_at: String,
    pub body: String,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct EntryRow {
    pub ts: String,
    pub kind: String,
    pub agent: String,
    pub claim_id: Option<String>,
    pub to_agent: Option<String>,
    pub body: String,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct PrRow {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub head_ref: String,
    pub state: String,
    pub draft: bool,
    pub checks_ok: u32,
    pub checks_fail: u32,
    pub checks_pending: u32,
    pub url: String,
    pub missing: Vec<String>,
}

/// A source with nothing behind it yet; lets the screen shell (PR 6) land before the data (PR 7).
pub struct StubSource;
impl Source for StubSource {
    fn snapshot(&mut self) -> Snapshot {
        Snapshot::default()
    }
    fn stop(&mut self, _pid: i64) -> std::result::Result<String, String> {
        Err("not implemented yet (plan PR 7)".into())
    }
    fn note(&mut self, _to: Option<String>, _body: String) -> std::result::Result<(), String> {
        Err("not implemented yet (plan PR 7)".into())
    }
    fn expire_claim(&mut self, _id: &str) -> std::result::Result<(), String> {
        Err("not implemented yet (plan PR 7)".into())
    }
}

/// Plain-text rendering used when stdout is not a terminal.
pub fn plain(s: &Snapshot) -> String {
    let mut out = format!("bf snapshot {}\n\nAGENTS ({})\n", s.at, s.agents.len());
    for a in &s.agents {
        out.push_str(&format!(
            "  {:<7} {:<7} {:<8} {}  {}\n",
            a.provider,
            a.pid,
            a.state,
            a.cwd,
            a.title.clone().unwrap_or_default()
        ));
    }
    out.push_str(&format!("\nCLAIMS ({})\n", s.claims.len()));
    for c in &s.claims {
        out.push_str(&format!(
            "  {} {} {} {} exp {} — {}\n",
            c.id,
            c.agent,
            c.repo,
            c.paths.join(","),
            c.expires_at,
            c.body
        ));
    }
    out.push_str(&format!("\nRECENT ({})\n", s.recent.len()));
    for e in &s.recent {
        out.push_str(&format!("  {} {} {} {}\n", e.ts, e.kind, e.agent, e.body));
    }
    out
}

pub fn run(mut source: Box<dyn Source>) -> Result<()> {
    let snapshot = source.snapshot();
    print!("{}", plain(&snapshot));
    Err(Error::Other(
        "not implemented yet (plan PR 6: the ratatui screen; PR 7: live data)".into(),
    ))
}
