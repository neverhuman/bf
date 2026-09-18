//! The screen's real data: sessions from `/proc` (`agents`), claims and entries from the board,
//! the load average, open PRs through `gh` (`prs`), each agent's transcript tail, and the digest
//! kept fresh while the screen runs. Nothing here panics: the first failure is logged to stderr
//! once and the snapshot degrades to what could be read.
use crate::agents::{self, Agent, Env};
use crate::board::{self, Board, Claim, Entry};
use crate::identity::{self, Identity};
use crate::prs::{self, Pr};
use crate::tui::text::terminal_text;
use crate::tui::{AgentRow, ClaimRow, EntryRow, PrRow, Snapshot, Source};
use crate::{runner, Result};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant, SystemTime};

const BOARD_FILE: &str = "bf.sqlite";
const DIGEST_FILE: &str = "AGENT_CHAT.md";
const RECENT: usize = 50;
/// How often the running screen rewrites `AGENT_CHAT.md` (its Live agents section).
const DIGEST_EVERY: Duration = Duration::from_secs(30);
/// How often the open-PR list is refetched through `gh`.
const PRS_EVERY: Duration = Duration::from_secs(60);
/// Only the first snapshot waits for the PR fetch, and at most this long, so a one-shot frame
/// (`BF_TUI_ONCE`, piped stdout) carries real counts while the loader stays under 2 s.
const PRS_FIRST_WAIT: Duration = Duration::from_millis(1500);
/// Bytes read from the end of a transcript.
const TAIL_BYTES: u64 = 64 * 1024;
/// Messages kept per agent, and characters kept per rendered message.
const TAIL_MESSAGES: usize = 40;
const TAIL_CHARS: usize = 200;
/// Transcript re-reads stop once one snapshot has spent this long on them; the agents left over
/// keep their cached tail and are read on the next snapshot.
const TAIL_BUDGET: Duration = Duration::from_millis(60);

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

/// One transcript's parsed tail, keyed by the file's (mtime, len) so an unchanged file is never
/// re-read.
struct Tail {
    mtime: Option<SystemTime>,
    len: u64,
    lines: Vec<String>,
}

/// A `Source` over the real host. The board is reopened per call so the loader thread never
/// holds a stale connection; discovery is passive (see `agents`). PRs come from a fetch thread
/// (`gh` may take seconds per repository) and are served from the last good result.
pub struct LiveSource {
    env: Env,
    board_path: PathBuf,
    digest_path: PathBuf,
    last_digest: Instant,
    last_prs: Vec<PrRow>,
    /// When the last fetch was started (the cadence) and the RFC3339 stamp of the last good
    /// one (the stale marker).
    last_prs_at: Option<Instant>,
    last_prs_stamp: Option<String>,
    prs_stale_since: Option<String>,
    /// The fetch in flight, if any.
    prs_pending: Option<Receiver<std::result::Result<Vec<Pr>, String>>>,
    /// The first fetch has answered (either way): later snapshots never wait on the channel.
    prs_settled: bool,
    tails: HashMap<PathBuf, Tail>,
}

impl LiveSource {
    pub fn from_env() -> Self {
        let dir = data_dir();
        Self {
            env: Env::from_env(),
            board_path: dir.join(BOARD_FILE),
            digest_path: dir.join(DIGEST_FILE),
            last_digest: digest_due(),
            last_prs: Vec::new(),
            last_prs_at: None,
            last_prs_stamp: None,
            prs_stale_since: None,
            prs_pending: None,
            prs_settled: false,
            tails: HashMap::new(),
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

    /// Start a fetch when one is due and none is running, then take whatever the fetch thread
    /// has answered. A good answer replaces the rows; a failure keeps them and marks them stale
    /// since the last good fetch (or `never`).
    fn refresh_prs(&mut self, now: DateTime<Utc>) {
        let due = self.last_prs_at.is_none_or(|t| t.elapsed() >= PRS_EVERY);
        if self.prs_pending.is_none() && due {
            let (tx, rx) = mpsc::channel();
            let board_path = self.board_path.clone();
            std::thread::spawn(move || {
                let repos = prs::repos_from_board(&board_path);
                let _ = tx.send(prs::list(&repos).map_err(|e| e.to_string()));
            });
            self.prs_pending = Some(rx);
            self.last_prs_at = Some(Instant::now());
        }
        let Some(rx) = &self.prs_pending else {
            return;
        };
        // Ok(answer) | Err(true): still running | Err(false): the fetch thread is gone.
        let answer = if self.prs_settled {
            rx.try_recv().map_err(|e| e == TryRecvError::Empty)
        } else {
            rx.recv_timeout(PRS_FIRST_WAIT)
                .map_err(|e| e == RecvTimeoutError::Timeout)
        };
        match answer {
            Err(true) => return,
            Err(false) => self.prs_stale_since = Some(self.last_good_prs()),
            Ok(Ok(prs)) => {
                self.last_prs = prs.into_iter().map(pr_row).collect();
                self.last_prs_stamp = Some(board::fmt_ts(now));
                self.prs_stale_since = None;
            }
            Ok(Err(_)) => self.prs_stale_since = Some(self.last_good_prs()),
        }
        self.prs_pending = None;
        self.prs_settled = true;
    }

    fn last_good_prs(&self) -> String {
        self.last_prs_stamp
            .clone()
            .unwrap_or_else(|| "never".to_string())
    }

    /// Every agent's transcript tail, re-read only for files whose (mtime, len) changed and only
    /// while this snapshot's read budget lasts; the rest keep their last tail. Entries for
    /// transcripts no agent has any more are dropped.
    fn transcript_tails(&mut self, agents: &[Agent]) -> Vec<Vec<String>> {
        let start = Instant::now();
        let mut keep: Vec<PathBuf> = Vec::new();
        let tails = agents
            .iter()
            .map(|a| {
                let Some(path) = transcript_path(&self.env, a) else {
                    return Vec::new();
                };
                let Ok(meta) = fs::metadata(&path) else {
                    return Vec::new();
                };
                let (mtime, len) = (meta.modified().ok(), meta.len());
                keep.push(path.clone());
                let cached = self.tails.get(&path);
                let fresh = cached.is_some_and(|t| t.mtime == mtime && t.len == len);
                if fresh || start.elapsed() >= TAIL_BUDGET {
                    return cached.map(|t| t.lines.clone()).unwrap_or_default();
                }
                let lines = read_tail(&path, len)
                    .map(|text| tail_lines(&text))
                    .unwrap_or_default();
                self.tails.insert(
                    path,
                    Tail {
                        mtime,
                        len,
                        lines: lines.clone(),
                    },
                );
                lines
            })
            .collect();
        self.tails.retain(|p, _| keep.contains(p));
        tails
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
        let tails = self.transcript_tails(&agents);
        self.refresh_prs(now);
        let mut snapshot = Snapshot {
            at: board::fmt_ts(now),
            agents: agents
                .iter()
                .zip(tails)
                .map(|(a, tail)| agent_row(&self.env.proc, a, now, tail))
                .collect(),
            prs: self.last_prs.clone(),
            prs_stale_since: self.prs_stale_since.clone(),
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

/// The file whose tail is the agent's conversation. Discovery hands Claude sessions their
/// registry row (`~/.claude/sessions/<pid>.json`), which is not a transcript; the transcript is
/// `~/.claude/projects/<cwd with every non-alphanumeric byte as '-'>/<session id>.jsonl`.
fn transcript_path(env: &Env, a: &Agent) -> Option<PathBuf> {
    let path = a.transcript_path.as_ref()?;
    if a.provider != "claude" || path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Some(path.clone());
    }
    let project: String = a
        .cwd
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    let jsonl = env
        .home
        .join(".claude/projects")
        .join(project)
        .join(format!("{}.jsonl", a.session_id.as_deref()?));
    jsonl.is_file().then_some(jsonl)
}

/// The last `TAIL_BYTES` of `path` (`len` from a fresh `metadata`), without the partial first
/// line when the file was longer than that.
fn read_tail(path: &Path, len: u64) -> std::io::Result<String> {
    let mut file = File::open(path)?;
    let start = len.saturating_sub(TAIL_BYTES);
    if start > 0 {
        file.seek(SeekFrom::Start(start))?;
    }
    let mut bytes = Vec::with_capacity(len.min(TAIL_BYTES) as usize);
    file.take(TAIL_BYTES).read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(match (start > 0, text.find('\n')) {
        (true, Some(nl)) => text[nl + 1..].to_owned(),
        (true, None) => String::new(),
        (false, _) => text.into_owned(),
    })
}

/// The last `TAIL_MESSAGES` messages of a JSONL transcript as `role: text`, oldest first, each
/// terminal-safe and at most `TAIL_CHARS` characters. Lines that are not JSON or not a message
/// (tool calls, thinking, metadata, injected `<…>` markup) are skipped.
pub fn tail_lines(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter_map(|v| message_of(&v))
        .map(|(role, text)| render(&role, &text))
        .collect();
    let extra = lines.len().saturating_sub(TAIL_MESSAGES);
    lines.drain(..extra);
    lines
}

/// `(role, text)` of one transcript line in any provider's shape:
/// Claude `{"type":"user"|"assistant","message":{"content":"…"|[{"text":…}]}}` (never `isMeta`),
/// Codex `{"type":"response_item","payload":{"type":"message","role":…,"content":[{"text":…}]}}`,
/// Cursor `{"role":…,"message":{"content":[{"text":…}]}}`,
/// Grok `{"type":"user"|"assistant","content":"…"}`.
fn message_of(v: &Value) -> Option<(String, String)> {
    let kind = v["type"].as_str();
    let (role, text) = if kind == Some("response_item") {
        let payload = &v["payload"];
        if payload["type"].as_str() != Some("message") {
            return None;
        }
        (payload["role"].as_str()?, text_of(&payload["content"])?)
    } else if let Some(role @ ("user" | "assistant")) = kind {
        if v["isMeta"].as_bool() == Some(true) {
            return None;
        }
        let text = match &v["message"] {
            Value::Object(message) => text_of(&message["content"])?,
            _ => v["content"].as_str()?.to_owned(),
        };
        (role, text)
    } else {
        (v["role"].as_str()?, text_of(&v["message"]["content"])?)
    };
    let text = text.trim();
    (!text.is_empty() && !text.starts_with('<')).then(|| (role.to_owned(), text.to_owned()))
}

/// A content string as is, or the `text` of every block that has one, joined by newlines.
fn text_of(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(blocks) => {
            let texts: Vec<&str> = blocks.iter().filter_map(|b| b["text"].as_str()).collect();
            (!texts.is_empty()).then(|| texts.join("\n"))
        }
        _ => None,
    }
}

fn render(role: &str, text: &str) -> String {
    let flat = terminal_text(text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut line = format!("{role}: {flat}");
    if line.chars().count() > TAIL_CHARS {
        line = line.chars().take(TAIL_CHARS - 1).collect();
        line.push('…');
    }
    line
}

fn agent_row(
    proc_root: &Path,
    a: &Agent,
    now: DateTime<Utc>,
    transcript_tail: Vec<String>,
) -> AgentRow {
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
        transcript_tail,
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

fn pr_row(p: Pr) -> PrRow {
    PrRow {
        repo: p.repo,
        number: p.number,
        title: p.title,
        head_ref: p.head_ref,
        state: p.state,
        draft: p.draft,
        checks_ok: p.checks_ok,
        checks_fail: p.checks_fail,
        checks_pending: p.checks_pending,
        url: p.url,
        missing: p.missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tail_lines_read_every_provider_shape() {
        let text = [
            // Claude: string content, block content; skipped: meta, `<…>` markup, tool_use only.
            r#"{"type":"user","message":{"role":"user","content":"fix the build"}}"#,
            r#"{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"hm"},{"type":"text","text":"On it.\nTwo  lines."},{"type":"tool_use","name":"Bash"}]}}"#,
            r#"{"type":"user","isMeta":true,"message":{"content":"Workflow authoring reference"}}"#,
            r#"{"type":"user","message":{"content":"<command-name>/model</command-name>"}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}"#,
            r#"{"type":"queue-operation","operation":"enqueue"}"#,
            // Codex: a message and a non-message item.
            r#"{"type":"response_item","payload":{"type":"message","role":"assistant","content":[{"type":"output_text","text":"The shared repository has changed."}]}}"#,
            r#"{"type":"response_item","payload":{"type":"reasoning","summary":[]}}"#,
            // Cursor.
            r#"{"role":"user","message":{"content":[{"type":"text","text":"inventory every checkout"}]}}"#,
            // Grok, one carrying an escape sequence and a bidi override.
            r#"{"type":"assistant","content":"\u001b[31mdone\u001b[0m \u202ecaps"}"#,
            "not json at all",
            "",
        ]
        .join("\n");
        assert_eq!(
            tail_lines(&text),
            vec![
                "user: fix the build",
                "assistant: On it. Two lines.",
                "assistant: The shared repository has changed.",
                "user: inventory every checkout",
                "assistant: done caps",
            ]
        );
    }

    #[test]
    fn tail_lines_keep_the_last_forty_and_truncate_each() {
        let long = "x".repeat(500);
        let lines: Vec<String> = (0..50)
            .map(|i| format!(r#"{{"type":"user","content":"{i} {long}"}}"#))
            .collect();
        let tail = tail_lines(&lines.join("\n"));
        assert_eq!(tail.len(), TAIL_MESSAGES);
        assert!(tail[0].starts_with("user: 10 x"), "{}", tail[0]);
        assert!(tail[39].starts_with("user: 49 x"), "{}", tail[39]);
        assert!(tail.iter().all(|l| l.chars().count() == TAIL_CHARS));
        assert!(tail.iter().all(|l| l.ends_with('…')));
    }

    #[test]
    fn read_tail_drops_the_partial_first_line_of_a_long_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.jsonl");
        let line = format!(r#"{{"type":"user","content":"{}"}}"#, "y".repeat(1000));
        let body = format!("{line}\n").repeat(80);
        fs::write(&path, &body).unwrap();
        let len = fs::metadata(&path).unwrap().len();
        assert!(len > TAIL_BYTES);
        let text = read_tail(&path, len).unwrap();
        assert!(text.starts_with(r#"{"type":"user""#), "{}", &text[..40]);
        assert!(text.len() < TAIL_BYTES as usize);
        assert_eq!(tail_lines(&text).len(), TAIL_MESSAGES);

        // A short file comes back whole, first line intact, whatever `len` claims.
        fs::write(&path, "{\"type\":\"user\",\"content\":\"a\"}\n").unwrap();
        let text = read_tail(&path, 1).unwrap();
        assert_eq!(tail_lines(&text), vec!["user: a"]);
    }

    #[test]
    fn transcript_path_resolves_claude_registry_rows_to_the_project_jsonl() {
        let home = tempfile::tempdir().unwrap();
        let env = Env {
            home: home.path().to_path_buf(),
            proc: home.path().join("proc"),
        };
        let registry = home.path().join(".claude/sessions/1.json");
        let mut a = Agent {
            provider: "claude".into(),
            session_id: Some("s-1".into()),
            cwd: "/home/u/a.b_c".into(),
            transcript_path: Some(registry.clone()),
            ..Agent::default()
        };
        assert_eq!(transcript_path(&env, &a), None, "no transcript yet");
        let jsonl = home.path().join(".claude/projects/-home-u-a-b-c/s-1.jsonl");
        fs::create_dir_all(jsonl.parent().unwrap()).unwrap();
        fs::write(&jsonl, "").unwrap();
        assert_eq!(transcript_path(&env, &a), Some(jsonl));
        a.provider = "codex".into();
        assert_eq!(transcript_path(&env, &a), Some(registry));
    }

    /// `cargo test --lib -- --ignored --nocapture live::tests::bench` on a real host: snapshots
    /// 2 s apart, as the screen takes them, until the PR fetch has landed (the first may wait for
    /// `gh`), then the tail reader over 32 fresh 64 KiB transcripts, the per-snapshot worst case
    /// for 30+ agents.
    #[test]
    #[ignore]
    fn bench_snapshot_and_tails() {
        let mut source = LiveSource::from_env();
        for i in 1..=6 {
            let start = Instant::now();
            let snapshot = source.snapshot();
            let landed = source.last_prs_stamp.is_some() || snapshot.prs_stale_since.is_some();
            eprintln!(
                "snapshot {i}: {:?} ({} agents, {} with a tail, {} PRs, stale {:?})",
                start.elapsed(),
                snapshot.agents.len(),
                snapshot
                    .agents
                    .iter()
                    .filter(|a| !a.transcript_tail.is_empty())
                    .count(),
                snapshot.prs.len(),
                snapshot.prs_stale_since
            );
            if landed {
                break;
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        let dir = tempfile::tempdir().unwrap();
        let line = format!(
            r#"{{"type":"assistant","message":{{"content":[{{"type":"text","text":"{}"}}]}}}}"#,
            "z".repeat(300)
        );
        let body = format!("{line}\n").repeat(250);
        let paths: Vec<PathBuf> = (0..32)
            .map(|i| {
                let p = dir.path().join(format!("{i}.jsonl"));
                fs::write(&p, &body).unwrap();
                p
            })
            .collect();
        let start = Instant::now();
        let n: usize = paths
            .iter()
            .map(|p| tail_lines(&read_tail(p, fs::metadata(p).unwrap().len()).unwrap()).len())
            .sum();
        eprintln!("32 transcript tails: {:?} ({n} lines)", start.elapsed());
    }
}
