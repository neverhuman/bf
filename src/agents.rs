//! Passive discovery of live coding-agent sessions (Claude, Codex, Cursor, Grok)
//! from `/proc` plus on-disk registries. No agent cooperation, no model calls.
use crate::identity;
use crate::{Error, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
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

pub fn discover(env: &Env) -> Result<Vec<Agent>> {
    let mut live: BTreeMap<i64, Agent> = BTreeMap::new();
    if env.proc.is_dir() {
        let entries = fs::read_dir(&env.proc).map_err(|e| Error::Other(e.to_string()))?;
        for entry in entries.flatten() {
            let Some(pid) = entry
                .file_name()
                .to_str()
                .and_then(|s| s.parse::<i64>().ok())
            else {
                continue;
            };
            if pid <= 0 {
                continue;
            }
            let dir = entry.path();
            let exe = fs::read_link(dir.join("exe"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let cmdline = read_nul(&dir.join("cmdline"));
            let comm = read_nul(&dir.join("comm"));
            let Some(provider) = classify_live(&exe, &cmdline, &comm) else {
                continue;
            };
            let (state, rss_kb) = status_fields(&dir.join("status"));
            let cwd = fs::read_link(dir.join("cwd"))
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            live.insert(
                pid,
                Agent {
                    provider: provider.to_string(),
                    pid,
                    session_id: resume_id(&cmdline),
                    state,
                    rss_kb,
                    cwd: cwd.clone(),
                    branch: git_branch(&cwd),
                    ..Agent::default()
                },
            );
        }
    }
    overlay_claude(env, &mut live);
    overlay_codex(env, &mut live);
    overlay_grok(env, &mut live);
    Ok(live.into_values().collect())
}

/// Cursor must match `cursor-agent/versions` so a shell whose argv only *mentions*
/// that string is not counted as a Cursor session.
fn classify_live<'a>(exe: &'a str, cmdline: &'a str, comm: &'a str) -> Option<&'static str> {
    if exe.contains("/.grok/") || comm == "grok" {
        return Some("grok");
    }
    if cmdline.contains("cursor-agent/versions/") {
        return Some("cursor");
    }
    identity::classify(exe, cmdline, comm).filter(|p| *p != "cursor")
}

fn overlay_claude(env: &Env, live: &mut BTreeMap<i64, Agent>) {
    let Ok(entries) = fs::read_dir(env.home.join(".claude/sessions")) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(v) = fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        else {
            continue;
        };
        let Some(pid) = v.get("pid").and_then(Value::as_i64) else {
            continue;
        };
        let Some(agent) = live.get_mut(&pid) else {
            continue; // registry row without a live process is dropped
        };
        if agent.provider != "claude" {
            continue;
        }
        agent.session_id = v
            .get("sessionId")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if let Some(cwd) = v.get("cwd").and_then(Value::as_str) {
            if agent.cwd.is_empty() {
                agent.cwd = cwd.to_owned();
                agent.branch = git_branch(cwd);
            }
        }
        agent.title = v.get("name").and_then(Value::as_str).map(str::to_owned);
        if let Some(status) = v.get("status").and_then(Value::as_str) {
            agent.state = status.to_owned();
        }
        agent.started_at = millis_field(&v, "startedAt");
        agent.last_activity_at = millis_field(&v, "updatedAt");
        agent.transcript_path = Some(path);
    }
}

fn overlay_codex(env: &Env, live: &mut BTreeMap<i64, Agent>) {
    let Ok(text) = fs::read_to_string(env.home.join(".codex/session_index.jsonl")) else {
        return;
    };
    let mut rows: Vec<(String, String, String)> = Vec::new();
    for line in text.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(id) = v.get("id").and_then(Value::as_str) else {
            continue;
        };
        rows.push((
            v.get("updated_at")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            id.to_owned(),
            v.get("thread_name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        ));
    }
    rows.sort();
    rows.reverse();
    let mut rows = rows.into_iter();
    for agent in live.values_mut().filter(|a| a.provider == "codex") {
        if agent.session_id.is_some() {
            continue;
        }
        if let Some((updated, id, title)) = rows.next() {
            agent.session_id = Some(id);
            if !title.is_empty() {
                agent.title = Some(title);
            }
            if !updated.is_empty() {
                agent.last_activity_at = Some(updated);
            }
        }
    }
}

fn overlay_grok(env: &Env, live: &mut BTreeMap<i64, Agent>) {
    let Ok(text) = fs::read_to_string(env.home.join(".grok/active_sessions.json")) else {
        return;
    };
    let Ok(rows) = serde_json::from_str::<Vec<Value>>(&text) else {
        return;
    };
    for row in rows {
        let Some(pid) = row.get("pid").and_then(Value::as_i64) else {
            continue;
        };
        let Some(agent) = live.get_mut(&pid) else {
            continue;
        };
        if agent.provider != "grok" {
            continue;
        }
        agent.session_id = row
            .get("session_id")
            .and_then(Value::as_str)
            .map(str::to_owned);
        if let Some(cwd) = row.get("cwd").and_then(Value::as_str) {
            if agent.cwd.is_empty() {
                agent.cwd = cwd.to_owned();
                agent.branch = git_branch(cwd);
            }
        }
        agent.started_at = row
            .get("opened_at")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
}

fn resume_id(cmdline: &str) -> Option<String> {
    let rest = cmdline.split("--resume=").nth(1)?;
    let id = rest.split_whitespace().next()?.trim();
    if id.is_empty() {
        None
    } else {
        Some(id.to_owned())
    }
}

fn read_nul(path: &Path) -> String {
    fs::read(path)
        .map(|b| {
            String::from_utf8_lossy(&b)
                .replace('\0', " ")
                .trim()
                .to_string()
        })
        .unwrap_or_default()
}

fn status_fields(status: &Path) -> (String, Option<u64>) {
    let text = fs::read_to_string(status).unwrap_or_default();
    let mut state = "running".to_string();
    let mut rss_kb = None;
    for line in text.lines() {
        if let Some(v) = line.strip_prefix("State:") {
            state = match v.trim().chars().next().unwrap_or('R') {
                'S' | 'I' => "idle".into(),
                'T' => "stopped".into(),
                'Z' => "zombie".into(),
                'D' => "waiting".into(),
                _ => "running".into(),
            };
        }
        if let Some(v) = line.strip_prefix("VmRSS:") {
            rss_kb = v.split_whitespace().next().and_then(|n| n.parse().ok());
        }
    }
    (state, rss_kb)
}

fn millis_field(v: &Value, key: &str) -> Option<String> {
    match v.get(key)? {
        Value::Number(n) => {
            chrono::DateTime::from_timestamp_millis(n.as_i64()?).map(|t| t.to_rfc3339())
        }
        Value::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn git_branch(cwd: &str) -> Option<String> {
    if cwd.is_empty() {
        return None;
    }
    let root = Path::new(cwd);
    if !root.join(".git").exists() {
        return None;
    }
    crate::gitutil::git(root, &["rev-parse", "--abbrev-ref", "HEAD"]).ok()
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
