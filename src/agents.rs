//! Passive discovery of live coding-agent sessions (Claude Code, Codex, Cursor, Grok) from their
//! on-disk registries and `/proc`. No agent cooperation, no model calls. Implemented in plan PR 4.
use crate::{Error, Result};
use serde::Serialize;
use std::path::PathBuf;

/// Where to look. `BF_HOME` and `BF_PROC` override `$HOME` and `/proc` (tests and CI only).
#[derive(Clone, Debug)]
pub struct Env {
    pub home: PathBuf,
    pub proc: PathBuf,
}
impl Env {
    pub fn from_env() -> Self {
        let home = std::env::var_os("BF_HOME")
            .or_else(|| std::env::var_os("HOME"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let proc = std::env::var_os("BF_PROC")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/proc"));
        Self { home, proc }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Agent {
    pub provider: String,
    pub pid: i64,
    pub session_id: Option<String>,
    pub state: String,
    pub waiting_for: Option<String>,
    pub started_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub age_secs: Option<u64>,
    pub rss_kb: Option<u64>,
    pub cwd: String,
    pub branch: Option<String>,
    pub title: Option<String>,
    pub last_prompt: Option<String>,
    pub transcript_path: Option<PathBuf>,
}

pub fn discover(_env: &Env) -> Result<Vec<Agent>> {
    Err(Error::Other(
        "not implemented yet (plan PR 4: agents.rs discovery)".into(),
    ))
}

pub fn plain_table(agents: &[Agent]) -> String {
    let mut s =
        String::from("PROVIDER PID     STATE    AGE   CWD                      BRANCH   TITLE\n");
    for a in agents {
        s.push_str(&format!(
            "{:<8} {:<7} {:<8} {:<5} {:<24} {:<8} {}\n",
            a.provider,
            a.pid,
            a.state,
            a.age_secs
                .map(|s| format!("{}m", s / 60))
                .unwrap_or_default(),
            a.cwd,
            a.branch.clone().unwrap_or_default(),
            a.title
                .clone()
                .or(a.last_prompt.clone())
                .unwrap_or_default()
        ));
    }
    s
}
