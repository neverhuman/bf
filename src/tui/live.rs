//! Live Source: `/proc` discovery plus the SQLite board. No model calls.
use super::{AgentRow, ClaimRow, EntryRow, Snapshot, Source};
use crate::agents::{self, Env};
use crate::board::{self, Board};
use crate::identity;
use chrono::Utc;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;

pub struct LiveSource {
    env: Env,
    data_dir: PathBuf,
}

impl LiveSource {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            env: Env::from_env(),
            data_dir,
        }
    }

    fn board(&self) -> Result<Board, String> {
        let _ = std::fs::create_dir_all(&self.data_dir);
        Board::open(&self.data_dir.join("bf.sqlite")).map_err(|e| e.to_string())
    }
}

impl Source for LiveSource {
    fn snapshot(&mut self) -> Snapshot {
        let now = Utc::now();
        let agents = agents::discover(&self.env).unwrap_or_default();
        let board = self.board().ok();
        let snap = board
            .as_ref()
            .and_then(|b| b.snapshot(now, 30).ok())
            .unwrap_or_default();
        Snapshot {
            at: board::fmt_ts(now),
            agents: agents
                .into_iter()
                .map(|a| AgentRow {
                    detail: json!({
                        "provider": a.provider,
                        "pid": a.pid,
                        "session_id": a.session_id,
                        "cwd": a.cwd,
                    }),
                    provider: a.provider,
                    pid: a.pid,
                    state: a.state,
                    waiting_for: a.waiting_for,
                    age_secs: a.age_secs,
                    cwd: a.cwd,
                    branch: a.branch,
                    title: a.title,
                    last_prompt: a.last_prompt,
                    transcript_tail: Vec::new(),
                })
                .collect(),
            claims: snap
                .claims
                .into_iter()
                .map(|c| ClaimRow {
                    id: c.id,
                    agent: c.agent,
                    provider: c.provider,
                    repo: c.repo,
                    paths: c.paths,
                    expires_at: c.expires_at,
                    body: c.body,
                })
                .collect(),
            recent: snap
                .recent
                .into_iter()
                .map(|e| EntryRow {
                    ts: e.ts,
                    kind: e.kind,
                    agent: e.agent,
                    claim_id: e.claim_id,
                    to_agent: e.to_agent,
                    body: e.body,
                })
                .collect(),
            prs: Vec::new(),
            prs_stale_since: Some("prs.rs not implemented".into()),
            load: loadavg(),
        }
    }

    fn stop(&mut self, pid: i64) -> std::result::Result<String, String> {
        if pid <= 1 || pid == std::process::id() as i64 {
            return Err("refusing to stop pid 1 or this bf process".into());
        }
        let status = Command::new("/bin/kill")
            .args(["-TERM", "--", &pid.to_string()])
            .status()
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err(format!("kill {pid} failed: {status}"));
        }
        Ok(format!("sent SIGTERM to {pid}"))
    }

    fn note(&mut self, to: Option<String>, body: String) -> std::result::Result<(), String> {
        let who = identity::current(None);
        let now = Utc::now();
        self.board()?
            .note(&who, &body, to.as_deref(), None, None, now)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    fn expire_claim(&mut self, claim_id: &str) -> std::result::Result<(), String> {
        let who = identity::current(None);
        let now = Utc::now();
        let mut board = self.board()?;
        board
            .release(&who, claim_id, "expired from TUI", now)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

fn loadavg() -> Option<f64> {
    std::fs::read_to_string("/proc/loadavg")
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}
