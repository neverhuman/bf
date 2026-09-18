//! The screen's real data: sessions from `/proc` (`agents`), claims and entries from the board,
//! the load average, and the digest kept fresh while the screen runs. Nothing here panics: the
//! first failure is logged to stderr once and the snapshot degrades to what could be read.
use crate::agents::{self, Agent, Env};
use crate::board::{self, Board, Claim, Entry};
use crate::identity::{self, Identity};
use crate::tui::{AgentRow, ClaimRow, EntryRow, Snapshot, Source};
use crate::{runner, Result};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const BOARD_FILE: &str = "bf.sqlite";
const DIGEST_FILE: &str = "AGENT_CHAT.md";
const RECENT: usize = 50;
/// How often the running screen rewrites `AGENT_CHAT.md` (its Live agents section).
const DIGEST_EVERY: Duration = Duration::from_secs(30);

static WARNED: AtomicBool = AtomicBool::new(false);

fn warn_once(what: &str, e: &dyn std::fmt::Display) {
    if !WARNED.swap(true, Ordering::Relaxed) {
        eprintln!("bf: {what}: {e}");
    }
}

/// `$BF_DATA_DIR`, else `$HOME/.bf` — the same resolution as the board verbs.
fn data_dir() -> PathBuf {
    std::env::var_os("BF_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".bf")
        })
}

/// A `Source` over the real host. The board is reopened per call so the loader thread never
/// holds a stale connection; discovery is passive (see `agents`).
pub struct LiveSource {
    env: Env,
    board_path: PathBuf,
    digest_path: PathBuf,
    last_digest: Instant,
}

impl LiveSource {
    pub fn from_env() -> Self {
        let dir = data_dir();
        Self {
            env: Env::from_env(),
            board_path: dir.join(BOARD_FILE),
            digest_path: dir.join(DIGEST_FILE),
            last_digest: digest_due(),
        }
    }

    fn operator() -> Identity {
        identity::current(None)
    }

    fn open(&self) -> Result<Board> {
        Board::open(&self.board_path)
    }

    /// Stop `pid`'s process group and record it on the board as the operator. Shared by
    /// `bf stop <pid>` and the screen's `x`.
    pub fn stop_recorded(&mut self, pid: i64) -> Result<String> {
        let target = agents::discover(&self.env)
            .ok()
            .and_then(|v| v.into_iter().find(|a| a.pid == pid));
        let message = runner::stop(pid)?;
        let (provider, cwd) = match target {
            Some(a) => (a.provider, a.cwd),
            None => ("process".to_string(), String::new()),
        };
        let body = if cwd.is_empty() {
            format!("stopped {provider} {pid}")
        } else {
            format!("stopped {provider} {pid} in {cwd}")
        };
        self.last_digest = digest_due();
        match self
            .open()
            .and_then(|mut b| b.record(&Self::operator(), "stop", &body, Utc::now()))
        {
            Ok(_) => Ok(message),
            Err(e) => Ok(format!("{message}; board record failed: {e}")),
        }
    }

    fn write_digest_if_due(&mut self, board: &mut Board, now: DateTime<Utc>, agents: &[Agent]) {
        if self.last_digest.elapsed() < DIGEST_EVERY {
            return;
        }
        self.last_digest = Instant::now();
        let section: String = agents
            .iter()
            .map(|a| {
                let line = format!(
                    "- {} {} {} {} {}",
                    a.provider,
                    a.pid,
                    a.state,
                    a.cwd,
                    a.title.as_deref().unwrap_or("")
                );
                format!("{}\n", line.trim_end())
            })
            .collect();
        if let Err(e) = board.write_digest(&self.digest_path, now, &section) {
            warn_once("digest", &e);
        }
    }
}

/// An instant far enough back that the next snapshot rewrites the digest.
fn digest_due() -> Instant {
    Instant::now()
        .checked_sub(DIGEST_EVERY)
        .unwrap_or_else(Instant::now)
}

impl Source for LiveSource {
    fn snapshot(&mut self) -> Snapshot {
        let now = Utc::now();
        let agents = match agents::discover(&self.env) {
            Ok(v) => v,
            Err(e) => {
                warn_once("discover", &e);
                Vec::new()
            }
        };
        let mut snapshot = Snapshot {
            at: board::fmt_ts(now),
            agents: agents
                .iter()
                .map(|a| agent_row(&self.env.proc, a, now))
                .collect(),
            load: load(&self.env.proc),
            ..Snapshot::default()
        };
        match self.open() {
            Ok(mut board) => {
                match board.active_claims(now) {
                    Ok(claims) => snapshot.claims = claims.into_iter().map(claim_row).collect(),
                    Err(e) => warn_once("board", &e),
                }
                match board.recent(RECENT) {
                    Ok(recent) => snapshot.recent = recent.into_iter().map(entry_row).collect(),
                    Err(e) => warn_once("board", &e),
                }
                self.write_digest_if_due(&mut board, now, &agents);
            }
            Err(e) => warn_once("board", &e),
        }
        snapshot
    }

    fn stop(&mut self, pid: i64) -> std::result::Result<String, String> {
        self.stop_recorded(pid).map_err(|e| e.to_string())
    }

    fn note(&mut self, to_agent: Option<String>, body: String) -> std::result::Result<(), String> {
        let mut board = self.open().map_err(|e| e.to_string())?;
        board
            .note(
                &Self::operator(),
                &body,
                to_agent.as_deref(),
                None,
                None,
                Utc::now(),
            )
            .map_err(|e| e.to_string())?;
        self.last_digest = digest_due();
        Ok(())
    }

    fn expire_claim(&mut self, claim_id: &str) -> std::result::Result<(), String> {
        let mut board = self.open().map_err(|e| e.to_string())?;
        board
            .release(
                &Self::operator(),
                claim_id,
                "expired by operator",
                Utc::now(),
            )
            .map_err(|e| e.to_string())?;
        self.last_digest = digest_due();
        Ok(())
    }
}

fn agent_row(proc_root: &Path, a: &Agent, now: DateTime<Utc>) -> AgentRow {
    AgentRow {
        provider: a.provider.clone(),
        pid: a.pid,
        state: a.state.clone(),
        waiting_for: a.waiting_for.clone(),
        age_secs: a.age_secs.or_else(|| age_secs(proc_root, a, now)),
        cwd: a.cwd.clone(),
        branch: a.branch.clone(),
        title: a.title.clone(),
        last_prompt: a.last_prompt.clone(),
        detail: serde_json::to_value(a).unwrap_or(serde_json::Value::Null),
        // No transcript reader exists yet (agents.rs exposes only the registry path).
        transcript_tail: Vec::new(),
    }
}

/// Seconds since the session started: the registry's `started_at` when the provider records
/// one, else the kernel's start time (`stat` field 22, in USER_HZ = 100 ticks since boot).
fn age_secs(proc_root: &Path, a: &Agent, now: DateTime<Utc>) -> Option<u64> {
    if let Some(started) = a
        .started_at
        .as_deref()
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
    {
        let secs = (now - started.with_timezone(&Utc)).num_seconds();
        return u64::try_from(secs).ok();
    }
    let uptime: f64 = std::fs::read_to_string(proc_root.join("uptime"))
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()?;
    let start_ticks: f64 = runner::stat_field(proc_root, a.pid, 22)?.parse().ok()?;
    let secs = uptime - start_ticks / 100.0;
    (secs.is_finite() && secs >= 0.0).then_some(secs as u64)
}

fn load(proc_root: &Path) -> Option<f64> {
    std::fs::read_to_string(proc_root.join("loadavg"))
        .ok()?
        .split_whitespace()
        .next()?
        .parse()
        .ok()
}

fn claim_row(c: Claim) -> ClaimRow {
    ClaimRow {
        id: c.id,
        agent: c.agent,
        provider: c.provider,
        repo: c.repo,
        paths: c.paths,
        expires_at: c.expires_at,
        body: c.body,
    }
}

fn entry_row(e: Entry) -> EntryRow {
    EntryRow {
        ts: e.ts,
        kind: e.kind,
        agent: e.agent,
        claim_id: e.claim_id,
        to_agent: e.to_agent,
        body: e.body,
    }
}
