//! Open pull requests across the repositories the board knows about, via the `gh` CLI.
//!
//! The repositories come from the board (active claims plus anything claimed in the last week)
//! and the current directory, mapped to `owner/name` through each repo's `origin` remote. Every
//! PR carries `missing`: the exact preconditions from AGENTS.md it has not met (green required
//! checks, a `REVIEW: approve <head-sha>` comment, `mergeStateStatus == CLEAN`, not a draft). An
//! empty `missing` means the PR is ready to merge.
use crate::board::Board;
use crate::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Instant;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Pr {
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
    /// Exact missing merge preconditions (CI, cross-vendor REVIEW comment, CLEAN).
    pub missing: Vec<String>,
}

/// How long one `gh pr list` may take before it is killed.
const GH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
/// Repositories touched on the board this recently count as "known".
const BOARD_WINDOW_DAYS: i64 = 7;
const GH_FIELDS: &str =
    "number,title,headRefName,headRefOid,isDraft,mergeStateStatus,url,statusCheckRollup,reviews,author";

/// `owner/name` from a GitHub remote URL, or `None` when the remote is not GitHub. Accepts
/// `git@github.com:o/n.git`, `ssh://git@github.com/o/n.git`, `https://github.com/o/n(.git)` and
/// scp-style SSH aliases such as `github-neverhuman:o/n.git`.
pub fn slug_from_origin(url: &str) -> Option<String> {
    let url = url.trim();
    let path = match url.split_once("://") {
        Some((scheme, rest)) => {
            if !matches!(scheme, "ssh" | "git+ssh" | "https" | "http" | "git") {
                return None;
            }
            let (host, path) = rest.split_once('/')?;
            if !is_github_host(host) {
                return None;
            }
            path
        }
        None => {
            let (host, path) = url.split_once(':')?;
            if host.contains('/') || !is_github_host(host) {
                return None;
            }
            path
        }
    };
    let path = path.trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let mut parts = path.split('/');
    let (owner, name) = (parts.next()?, parts.next()?);
    if owner.is_empty() || name.is_empty() || parts.next().is_some() {
        return None;
    }
    Some(format!("{owner}/{name}"))
}

/// `github.com` (with an optional `user@` and `:port`) or an SSH alias starting with `github`.
fn is_github_host(host: &str) -> bool {
    let host = host.rsplit('@').next().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host).to_ascii_lowercase();
    host == "github.com" || host.starts_with("github")
}

/// The `origin` slug of the repository at `dir`, if it is a git repository with a GitHub origin.
fn origin_slug(dir: &Path) -> Option<String> {
    let url = crate::gitutil::git(dir, &["remote", "get-url", "origin"]).ok()?;
    slug_from_origin(&url)
}

/// Distinct, sorted `owner/name` slugs: every repository behind an active claim or any board entry
/// in the last week, plus the current directory's repository. Repositories that are not git or
/// have no GitHub origin are skipped; a missing or unreadable board only leaves the cwd.
pub fn repos_from_board(board_path: &Path) -> Vec<String> {
    let mut dirs = BTreeSet::new();
    if board_path.is_file() {
        match board_dirs(board_path) {
            Ok(found) => dirs.extend(found),
            Err(e) => eprintln!("bf: board unreadable, using the cwd only: {e}"),
        }
    }
    let mut slugs: BTreeSet<String> = dirs
        .iter()
        .filter_map(|d| origin_slug(Path::new(d)))
        .collect();
    if let Some(slug) = std::env::current_dir()
        .ok()
        .and_then(|cwd| origin_slug(&cwd))
    {
        slugs.insert(slug);
    }
    slugs.into_iter().collect()
}

fn board_dirs(board_path: &Path) -> Result<BTreeSet<String>> {
    let board = Board::open(board_path)?;
    let now = Utc::now();
    let since = now - Duration::days(BOARD_WINDOW_DAYS);
    let mut dirs: BTreeSet<String> = board
        .active_claims(now)?
        .into_iter()
        .map(|c| c.repo)
        .collect();
    for entry in board.all()? {
        let Some(repo) = entry.repo else { continue };
        let recent = DateTime::parse_from_rfc3339(&entry.ts)
            .map(|t| t.with_timezone(&Utc) >= since)
            .unwrap_or(false);
        if recent {
            dirs.insert(repo);
        }
    }
    Ok(dirs)
}

/// Open PRs of every slug, oldest number first within a repo. A repository whose `gh` call fails
/// is skipped; when every repository fails the first failure is returned as `gh unavailable`.
pub fn list(repos: &[String]) -> Result<Vec<Pr>> {
    let mut prs = Vec::new();
    let mut first_failure = None;
    let mut successes = 0usize;
    for slug in repos {
        match gh_pr_list(slug) {
            Ok(values) => {
                successes += 1;
                prs.extend(values.iter().map(|v| from_value(slug, v)));
            }
            Err(e) => {
                if first_failure.is_none() {
                    first_failure = Some(e);
                }
            }
        }
    }
    match first_failure {
        Some(e) if successes == 0 => Err(Error::Other(format!("gh unavailable: {e}"))),
        _ => Ok(prs),
    }
}

/// `gh pr list -R <slug> …` with a hard deadline; `Err` carries the first line of `gh`'s complaint.
fn gh_pr_list(slug: &str) -> std::result::Result<Vec<Value>, String> {
    let mut child = Command::new("gh")
        .args(["pr", "list", "-R", slug, "--state", "open", "--limit", "50"])
        .args(["--json", GH_FIELDS])
        .env("GH_PROMPT_DISABLED", "1")
        .env("GH_NO_UPDATE_NOTIFIER", "1")
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("gh: {e}"))?;
    let stdout = child
        .stdout
        .take()
        .map(|p| std::thread::spawn(move || read_all(p)));
    let stderr = child
        .stderr
        .take()
        .map(|p| std::thread::spawn(move || read_all(p)));
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if start.elapsed() > GH_TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("gh pr list -R {slug}: no answer after 10s"));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(20)),
            Err(e) => return Err(format!("gh: {e}")),
        }
    };
    let join = |h: Option<std::thread::JoinHandle<Vec<u8>>>| {
        h.and_then(|h| h.join().ok()).unwrap_or_default()
    };
    let (out, err) = (join(stdout), join(stderr));
    if !status.success() {
        let line = String::from_utf8_lossy(&err)
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("gh pr list -R {slug}: {status}"));
        return Err(line);
    }
    let value: Value = serde_json::from_slice(&out)
        .map_err(|e| format!("gh pr list -R {slug}: not JSON ({e})"))?;
    Ok(value.as_array().cloned().unwrap_or_default())
}

fn read_all(mut pipe: impl Read) -> Vec<u8> {
    let mut buf = Vec::new();
    let _ = pipe.read_to_end(&mut buf);
    buf
}

/// One PR from a `gh pr list --json` element. Tolerant of missing fields; `missing` lists every
/// AGENTS.md merge precondition the PR has not met, in the order a reviewer would fix them.
pub fn from_value(repo: &str, v: &Value) -> Pr {
    let text = |key: &str| v[key].as_str().unwrap_or("").to_owned();
    let head_sha = text("headRefOid").to_ascii_lowercase();
    let (mut ok, mut fail, mut pending) = (0u32, 0u32, 0u32);
    for check in v["statusCheckRollup"].as_array().into_iter().flatten() {
        // CheckRun carries `conclusion` (null while running); StatusContext carries `state`.
        let verdict = check["conclusion"]
            .as_str()
            .or_else(|| check["state"].as_str())
            .unwrap_or("")
            .to_ascii_uppercase();
        match verdict.as_str() {
            "SUCCESS" | "NEUTRAL" | "SKIPPED" => ok += 1,
            "FAILURE" | "CANCELLED" | "TIMED_OUT" | "ERROR" | "ACTION_REQUIRED"
            | "STARTUP_FAILURE" | "STALE" => fail += 1,
            _ => pending += 1,
        }
    }
    let approved = v["reviews"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| r["body"].as_str())
        .any(|body| approves(body, &head_sha));
    let state = match v["mergeStateStatus"].as_str() {
        Some(s) if !s.is_empty() => s.to_owned(),
        _ => "UNKNOWN".to_owned(),
    };
    let draft = v["isDraft"].as_bool().unwrap_or(false);
    let mut missing = Vec::new();
    if fail > 0 {
        missing.push(format!("CI: {fail} failing"));
    }
    if pending > 0 {
        missing.push(format!("CI: {pending} pending"));
    }
    if !approved {
        let sha = if head_sha.is_empty() {
            "<head-sha>"
        } else {
            head_sha.as_str()
        };
        missing.push(format!("review: no REVIEW: approve {sha} comment"));
    }
    if state != "CLEAN" {
        missing.push(format!("mergeStateStatus: {state}"));
    }
    if draft {
        missing.push("draft".to_owned());
    }
    Pr {
        repo: repo.to_owned(),
        number: v["number"].as_u64().unwrap_or(0),
        title: text("title"),
        head_ref: text("headRefName"),
        state,
        draft,
        checks_ok: ok,
        checks_fail: fail,
        checks_pending: pending,
        url: text("url"),
        missing,
    }
}

/// `body` contains `REVIEW: approve <sha>` where `<sha>` is the head sha or a prefix of at least
/// seven hex characters.
fn approves(body: &str, head_sha: &str) -> bool {
    if head_sha.is_empty() {
        return false;
    }
    body.match_indices("REVIEW: approve ").any(|(i, marker)| {
        let sha: String = body[i + marker.len()..]
            .chars()
            .take_while(|c| c.is_ascii_hexdigit())
            .collect::<String>()
            .to_ascii_lowercase();
        sha.len() >= 7 && head_sha.starts_with(&sha)
    })
}

/// Plain ASCII table: one header, then one line per PR ending in `-> ready` or `-> missing: …`.
pub fn plain_table(prs: &[Pr]) -> String {
    let id = |p: &Pr| format!("{}#{}", p.repo, p.number);
    let width = prs.iter().map(|p| id(p).len()).max().unwrap_or(0).max(4);
    let mut s = format!("{:<width$}  {:<9} {:<24} TITLE\n", "PR", "STATE", "CHECKS");
    for p in prs {
        let checks = format!(
            "ok={} fail={} pending={}",
            p.checks_ok, p.checks_fail, p.checks_pending
        );
        let draft = if p.draft { "[draft] " } else { "" };
        let verdict = if p.missing.is_empty() {
            "ready".to_owned()
        } else {
            format!("missing: {}", p.missing.join("; "))
        };
        s.push_str(&format!(
            "{:<width$}  {:<9} {:<24} {draft}{} ({})  -> {verdict}\n",
            id(p),
            p.state,
            checks,
            p.title.replace('\n', " "),
            p.head_ref
        ));
    }
    s
}
