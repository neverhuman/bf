//! The coordination board that replaces hand-edited AGENT_CHAT.md.
//!
//! One append-only SQLite table. Claims reserve repo-relative paths for a bounded time; heartbeats
//! extend them; releases end them with proof; notes are addressed to an agent, a claim or a PR.
//! Every read derives the active set from the log, so expiry needs no background loop.
//!
//! The `cli_*` functions at the bottom are what `main.rs` calls: they resolve the data directory
//! (`$BF_DATA_DIR`, else `$HOME/.bf`), the writer identity and the repository, and regenerate
//! `AGENT_CHAT.md` after every write.
use crate::identity::{self, Identity};
use crate::{Error, Result};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS board(
  seq INTEGER PRIMARY KEY AUTOINCREMENT,
  ts TEXT NOT NULL,
  agent TEXT NOT NULL,
  provider TEXT NOT NULL CHECK(provider IN('claude','codex','cursor','grok','human','unknown','legacy-md')),
  pid INTEGER,
  kind TEXT NOT NULL CHECK(kind IN('claim','heartbeat','release','note','stop','run')),
  claim_id TEXT,
  repo TEXT,
  paths_json TEXT NOT NULL DEFAULT '[]',
  to_agent TEXT,
  pr_url TEXT,
  body TEXT NOT NULL CHECK(length(body) <= 2000),
  expires_at TEXT
);
CREATE INDEX IF NOT EXISTS board_claim ON board(claim_id, kind, seq);
CREATE INDEX IF NOT EXISTS board_to ON board(to_agent, seq);
CREATE TRIGGER IF NOT EXISTS board_immutable_u BEFORE UPDATE ON board BEGIN SELECT RAISE(ABORT,'immutable_board'); END;
CREATE TRIGGER IF NOT EXISTS board_immutable_d BEFORE DELETE ON board BEGIN SELECT RAISE(ABORT,'immutable_board'); END;
CREATE TABLE IF NOT EXISTS jobs(
  id TEXT PRIMARY KEY, provider TEXT NOT NULL, cwd TEXT NOT NULL, argv_json TEXT NOT NULL,
  pid INTEGER, pgid INTEGER, log_path TEXT NOT NULL, started_at TEXT NOT NULL, ended_at TEXT, exit_code INTEGER
);
CREATE TABLE IF NOT EXISTS inbox_seen(agent TEXT PRIMARY KEY, seq INTEGER NOT NULL);
"#;

pub const DEFAULT_TTL_MINUTES: i64 = 30;
pub const MAX_TTL_MINUTES: i64 = 180;
pub const STATUS_NOTE_MAX: usize = 280;
pub const END_MARKER: &str = "<!-- bf:end -->";
const RECENT_IN_DIGEST: usize = 30;
const COLS: &str =
    "seq,ts,agent,provider,pid,kind,claim_id,repo,paths_json,to_agent,pr_url,body,expires_at";

pub fn fmt_ts(t: DateTime<Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Entry {
    pub seq: i64,
    pub ts: String,
    pub agent: String,
    pub provider: String,
    pub pid: Option<i64>,
    pub kind: String,
    pub claim_id: Option<String>,
    pub repo: Option<String>,
    pub paths: Vec<String>,
    pub to_agent: Option<String>,
    pub pr_url: Option<String>,
    pub body: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct Claim {
    pub id: String,
    pub agent: String,
    pub provider: String,
    pub pid: Option<i64>,
    pub repo: String,
    pub paths: Vec<String>,
    pub body: String,
    pub claimed_at: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum ClaimOutcome {
    Claimed(Claim),
    AlreadyHeld(Claim),
}

#[derive(Clone, Debug, Serialize, Default)]
pub struct Snapshot {
    pub at: String,
    pub claims: Vec<Claim>,
    pub recent: Vec<Entry>,
}

pub struct Board {
    conn: Connection,
    pub path: PathBuf,
}

/// Validate and normalise one claimed path: repo-relative, no traversal, a trailing `/` marks a
/// directory; `dir/**` and `dir/*` are accepted as `dir/`; `.` means the whole repository.
pub fn normalize_path(raw: &str) -> Result<String> {
    let p = raw.trim();
    if p.is_empty() || p.contains('\0') || p.contains('\\') || p.starts_with('/') {
        return Err(Error::InvalidContract(format!(
            "path {raw:?} must be repo-relative with forward slashes"
        )));
    }
    let p = p
        .strip_suffix("/**")
        .or_else(|| p.strip_suffix("/*"))
        .map(|d| format!("{d}/"))
        .unwrap_or_else(|| p.to_string());
    if p == "." || p == "./" {
        return Ok(".".into());
    }
    let p = p.strip_prefix("./").unwrap_or(&p).to_string();
    let is_dir = p.ends_with('/');
    let parts: Vec<&str> = p.trim_end_matches('/').split('/').collect();
    if parts
        .iter()
        .any(|c| c.is_empty() || *c == "." || *c == "..")
    {
        return Err(Error::InvalidContract(format!(
            "path {raw:?} has an empty, `.` or `..` component"
        )));
    }
    if p.contains('*') {
        return Err(Error::InvalidContract(format!(
            "path {raw:?}: globs are not supported; claim a directory with a trailing slash"
        )));
    }
    Ok(if is_dir {
        format!("{}/", parts.join("/"))
    } else {
        parts.join("/")
    })
}

fn components(p: &str) -> Vec<&str> {
    if p == "." {
        return Vec::new();
    }
    p.trim_end_matches('/').split('/').collect()
}

/// Two normalised paths overlap when one is a component-wise prefix of the other.
pub fn overlaps(a: &str, b: &str) -> bool {
    let (ca, cb) = (components(a), components(b));
    let n = ca.len().min(cb.len());
    ca[..n] == cb[..n]
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|t| t.with_timezone(&Utc))
        .map_err(|e| Error::StorageUnavailable(format!("bad timestamp {s:?}: {e}")))
}

impl Board {
    pub fn open(path: &Path) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let conn = Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_millis(2000))?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;",
        )?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    /// Run `f` inside an IMMEDIATE transaction, retrying a few times on SQLITE_BUSY.
    fn write<T>(&mut self, mut f: impl FnMut(&rusqlite::Transaction) -> Result<T>) -> Result<T> {
        let mut attempt = 0;
        loop {
            match self
                .conn
                .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)
            {
                Ok(tx) => {
                    let v = f(&tx)?;
                    tx.commit()?;
                    return Ok(v);
                }
                Err(e) if attempt < 3 && e.to_string().contains("locked") => {
                    attempt += 1;
                    std::thread::sleep(std::time::Duration::from_millis(
                        50 * attempt + (attempt * 37) % 60,
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn insert(
        tx: &rusqlite::Transaction,
        now: DateTime<Utc>,
        who: &Identity,
        kind: &str,
        claim_id: Option<&str>,
        repo: Option<&str>,
        paths: &[String],
        to_agent: Option<&str>,
        pr_url: Option<&str>,
        body: &str,
        expires_at: Option<&str>,
    ) -> Result<i64> {
        if body.len() > 2000 {
            return Err(Error::InvalidContract("body exceeds 2000 bytes".into()));
        }
        tx.execute(
            "INSERT INTO board(ts,agent,provider,pid,kind,claim_id,repo,paths_json,to_agent,pr_url,body,expires_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
            params![fmt_ts(now), who.agent, who.provider, who.pid, kind, claim_id, repo, serde_json::to_string(paths)?, to_agent, pr_url, body, expires_at],
        )?;
        Ok(tx.last_insert_rowid())
    }

    fn active_claims_in(conn: &Connection, now: DateTime<Utc>) -> Result<Vec<Claim>> {
        let mut stmt = conn.prepare(
            "SELECT c.claim_id, c.agent, c.provider, c.pid, c.repo, c.paths_json, c.body, c.ts,
                    COALESCE((SELECT h.expires_at FROM board h WHERE h.claim_id=c.claim_id AND h.kind='heartbeat' ORDER BY h.seq DESC LIMIT 1), c.expires_at)
             FROM board c
             WHERE c.kind='claim'
               AND NOT EXISTS (SELECT 1 FROM board r WHERE r.claim_id=c.claim_id AND r.kind='release')
             ORDER BY c.seq",
        )?;
        let now_s = fmt_ts(now);
        let rows = stmt.query_map([], |r| {
            Ok(Claim {
                id: r.get(0)?,
                agent: r.get(1)?,
                provider: r.get(2)?,
                pid: r.get(3)?,
                repo: r.get(4)?,
                paths: serde_json::from_str(&r.get::<_, String>(5)?).unwrap_or_default(),
                body: r.get(6)?,
                claimed_at: r.get(7)?,
                expires_at: r.get(8)?,
            })
        })?;
        let mut out = Vec::new();
        for c in rows {
            let c = c?;
            if c.expires_at > now_s {
                out.push(c);
            }
        }
        Ok(out)
    }

    pub fn active_claims(&self, now: DateTime<Utc>) -> Result<Vec<Claim>> {
        Self::active_claims_in(&self.conn, now)
    }

    pub fn claim(
        &mut self,
        who: &Identity,
        repo: &str,
        raw_paths: &[String],
        body: &str,
        ttl_minutes: i64,
        now: DateTime<Utc>,
    ) -> Result<ClaimOutcome> {
        if raw_paths.is_empty() {
            return Err(Error::InvalidContract("claim at least one path".into()));
        }
        if body.trim().is_empty() {
            return Err(Error::InvalidContract("say what and why with -m".into()));
        }
        if !(1..=MAX_TTL_MINUTES).contains(&ttl_minutes) {
            return Err(Error::InvalidContract(format!(
                "ttl must be 1..={MAX_TTL_MINUTES} minutes"
            )));
        }
        let mut paths: Vec<String> = raw_paths
            .iter()
            .map(|p| normalize_path(p))
            .collect::<Result<_>>()?;
        paths.sort();
        paths.dedup();
        let who = who.clone();
        let body = body.to_string();
        let repo = repo.to_string();
        self.write(move |tx| {
            let active = Self::active_claims_in(tx, now)?;
            for c in active.iter().filter(|c| c.repo == repo) {
                let same_set = c.paths == paths;
                if same_set && c.agent == who.agent {
                    return Ok(ClaimOutcome::AlreadyHeld(c.clone()));
                }
                if let Some((mine, theirs)) = paths.iter().find_map(|p| {
                    c.paths
                        .iter()
                        .find(|q| overlaps(p, q))
                        .map(|q| (p.clone(), q.clone()))
                }) {
                    let hint = if c.agent == who.agent {
                        " (your own claim: heartbeat or release it first)"
                    } else {
                        ""
                    };
                    return Err(Error::ResourceConflict(format!(
                        "{mine} overlaps {theirs} held by {} as {} until {} — \"{}\"{hint}",
                        c.agent, c.id, c.expires_at, c.body
                    )));
                }
            }
            let next: i64 =
                tx.query_row("SELECT COALESCE(MAX(seq),0)+1 FROM board", [], |r| r.get(0))?;
            let id = format!("c-{next}");
            let expires = fmt_ts(now + Duration::minutes(ttl_minutes));
            Self::insert(
                tx,
                now,
                &who,
                "claim",
                Some(&id),
                Some(&repo),
                &paths,
                None,
                None,
                &body,
                Some(&expires),
            )?;
            Ok(ClaimOutcome::Claimed(Claim {
                id,
                agent: who.agent.clone(),
                provider: who.provider.clone(),
                pid: who.pid,
                repo: repo.clone(),
                paths: paths.clone(),
                body: body.clone(),
                claimed_at: fmt_ts(now),
                expires_at: expires,
            }))
        })
    }

    fn require_active(
        tx: &rusqlite::Transaction,
        now: DateTime<Utc>,
        claim_id: &str,
    ) -> Result<Claim> {
        Self::active_claims_in(tx, now)?
            .into_iter()
            .find(|c| c.id == claim_id)
            .ok_or(Error::StaleVersion)
    }

    pub fn heartbeat(
        &mut self,
        who: &Identity,
        claim_id: &str,
        body: &str,
        ttl_minutes: Option<i64>,
        now: DateTime<Utc>,
    ) -> Result<Claim> {
        let who = who.clone();
        let (claim_id, body) = (claim_id.to_string(), body.to_string());
        self.write(move |tx| {
            let c = Self::require_active(tx, now, &claim_id)?;
            let ttl = ttl_minutes.unwrap_or_else(|| {
                parse_ts(&c.expires_at)
                    .ok()
                    .zip(parse_ts(&c.claimed_at).ok())
                    .map(|(e, s)| (e - s).num_minutes())
                    .filter(|m| *m > 0)
                    .unwrap_or(DEFAULT_TTL_MINUTES)
            });
            let ttl = ttl.clamp(1, MAX_TTL_MINUTES);
            let expires = fmt_ts(now + Duration::minutes(ttl));
            Self::insert(
                tx,
                now,
                &who,
                "heartbeat",
                Some(&claim_id),
                Some(&c.repo),
                &c.paths,
                None,
                None,
                &body,
                Some(&expires),
            )?;
            Ok(Claim {
                expires_at: expires,
                ..c
            })
        })
    }

    pub fn release(
        &mut self,
        who: &Identity,
        claim_id: &str,
        body: &str,
        now: DateTime<Utc>,
    ) -> Result<Claim> {
        let who = who.clone();
        let (claim_id, body) = (claim_id.to_string(), body.to_string());
        self.write(move |tx| {
            let c = Self::require_active(tx, now, &claim_id)?;
            Self::insert(
                tx,
                now,
                &who,
                "release",
                Some(&claim_id),
                Some(&c.repo),
                &c.paths,
                None,
                None,
                &body,
                None,
            )?;
            Ok(c)
        })
    }

    /// An addressed note (to an agent, about a claim, or about a PR) may be up to 2000 bytes; an
    /// unaddressed status line is capped at `STATUS_NOTE_MAX`.
    pub fn note(
        &mut self,
        who: &Identity,
        body: &str,
        to_agent: Option<&str>,
        re_claim: Option<&str>,
        pr_url: Option<&str>,
        now: DateTime<Utc>,
    ) -> Result<i64> {
        if body.trim().is_empty() {
            return Err(Error::InvalidContract("empty note".into()));
        }
        if to_agent.is_none()
            && re_claim.is_none()
            && pr_url.is_none()
            && body.len() > STATUS_NOTE_MAX
        {
            return Err(Error::InvalidContract(format!(
                "an unaddressed status note is limited to {STATUS_NOTE_MAX} bytes; address it with --to, --re or --pr"
            )));
        }
        let who = who.clone();
        let body = body.to_string();
        let (to, re, pr) = (
            to_agent.map(String::from),
            re_claim.map(String::from),
            pr_url.map(String::from),
        );
        self.write(move |tx| {
            Self::insert(
                tx,
                now,
                &who,
                "note",
                re.as_deref(),
                None,
                &[],
                to.as_deref(),
                pr.as_deref(),
                &body,
                None,
            )
        })
    }

    /// Record an operator or runner event (`stop`, `run`).
    pub fn record(
        &mut self,
        who: &Identity,
        kind: &str,
        body: &str,
        now: DateTime<Utc>,
    ) -> Result<i64> {
        if kind != "stop" && kind != "run" {
            return Err(Error::InvalidContract(format!(
                "unknown record kind {kind}"
            )));
        }
        let who = who.clone();
        let (kind, body) = (kind.to_string(), body.to_string());
        self.write(move |tx| {
            Self::insert(
                tx,
                now,
                &who,
                &kind,
                None,
                None,
                &[],
                None,
                None,
                &body,
                None,
            )
        })
    }

    fn rows(&self, sql: &str, p: &[&dyn rusqlite::ToSql]) -> Result<Vec<Entry>> {
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map(p, |r| {
            Ok(Entry {
                seq: r.get(0)?,
                ts: r.get(1)?,
                agent: r.get(2)?,
                provider: r.get(3)?,
                pid: r.get(4)?,
                kind: r.get(5)?,
                claim_id: r.get(6)?,
                repo: r.get(7)?,
                paths: serde_json::from_str(&r.get::<_, String>(8)?).unwrap_or_default(),
                to_agent: r.get(9)?,
                pr_url: r.get(10)?,
                body: r.get(11)?,
                expires_at: r.get(12)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn recent(&self, limit: usize) -> Result<Vec<Entry>> {
        let mut v = self.rows(
            &format!("SELECT {COLS} FROM board ORDER BY seq DESC LIMIT ?1"),
            &[&(limit as i64)],
        )?;
        v.reverse();
        Ok(v)
    }
    pub fn all(&self) -> Result<Vec<Entry>> {
        self.rows(&format!("SELECT {COLS} FROM board ORDER BY seq"), &[])
    }
    /// Notes addressed to `agent`, newest last, optionally only those after `after_seq`.
    pub fn inbox(&self, agent: &str, after_seq: i64) -> Result<Vec<Entry>> {
        self.rows(
            &format!(
                "SELECT {COLS} FROM board WHERE kind='note' AND to_agent=?1 AND seq>?2 ORDER BY seq"
            ),
            &[&agent, &after_seq],
        )
    }
    pub fn last_seq(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COALESCE(MAX(seq),0) FROM board", [], |r| r.get(0))?)
    }
    /// The last board seq `agent` has read their inbox up to (0 = never).
    pub fn inbox_seen(&self, agent: &str) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(seq),0) FROM inbox_seen WHERE agent=?1",
            [agent],
            |r| r.get(0),
        )?)
    }
    pub fn mark_inbox_seen(&mut self, agent: &str, seq: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO inbox_seen(agent,seq) VALUES(?1,?2) ON CONFLICT(agent) DO UPDATE SET seq=excluded.seq",
            params![agent, seq],
        )?;
        Ok(())
    }
    pub fn snapshot(&self, now: DateTime<Utc>, recent: usize) -> Result<Snapshot> {
        Ok(Snapshot {
            at: fmt_ts(now),
            claims: self.active_claims(now)?,
            recent: self.recent(recent)?,
        })
    }

    /// Render the generated AGENT_CHAT.md. `agents_section` is supplied by discovery (may be empty).
    pub fn render_digest(&self, now: DateTime<Utc>, agents_section: &str) -> Result<String> {
        let snap = self.snapshot(now, RECENT_IN_DIGEST)?;
        let mut s = String::new();
        s.push_str(&format!(
            "# BulletFarm board — GENERATED by bf at {} · do not edit\n\n",
            snap.at
        ));
        s.push_str("Post with: `bf claim <paths> -m \"why\"` · `bf heartbeat <id>` · `bf release <id> --proof '<cmd>'` · `bf note -m \"…\" --to <agent>|--re <id>|--pr <url>` · read with `bf board` / `bf board --to me`.\n\n");
        s.push_str(&format!("## Active claims ({})\n\n", snap.claims.len()));
        for c in &snap.claims {
            s.push_str(&format!(
                "- `{}` {} ({}{}) `{}`: {} — exp {} — {}\n",
                c.id,
                c.agent,
                c.provider,
                c.pid.map(|p| format!("/{p}")).unwrap_or_default(),
                c.repo,
                c.paths.join(", "),
                c.expires_at,
                c.body.replace('\n', " ")
            ));
        }
        if !agents_section.trim().is_empty() {
            s.push_str("\n## Live agents\n\n");
            s.push_str(agents_section.trim_end());
            s.push('\n');
        }
        s.push_str(&format!("\n## Recent (last {})\n\n", RECENT_IN_DIGEST));
        for e in &snap.recent {
            let target = e
                .to_agent
                .as_ref()
                .map(|t| format!(" → {t}"))
                .or_else(|| e.pr_url.clone().map(|u| format!(" ⟶ {u}")))
                .unwrap_or_default();
            let cid = e
                .claim_id
                .as_ref()
                .map(|c| format!(" {c}"))
                .unwrap_or_default();
            s.push_str(&format!(
                "- {} {} {}{}{}: {}\n",
                e.ts,
                e.kind,
                e.agent,
                cid,
                target,
                e.body.replace('\n', " ")
            ));
        }
        s.push_str(&format!("\n{END_MARKER}\n"));
        Ok(s)
    }

    /// Import any hand-written lines found after the end marker of an existing digest as notes.
    pub fn import_stragglers(&mut self, existing: &str, now: DateTime<Utc>) -> Result<usize> {
        let Some(idx) = existing.find(END_MARKER) else {
            return Ok(0);
        };
        let tail = &existing[idx + END_MARKER.len()..];
        let who = Identity {
            agent: "legacy-md".into(),
            provider: "legacy-md".into(),
            pid: None,
        };
        let mut n = 0;
        for line in tail.lines().map(str::trim).filter(|l| !l.is_empty()) {
            let body: String = line.chars().take(1990).collect();
            self.write(|tx| {
                Self::insert(
                    tx,
                    now,
                    &who,
                    "note",
                    None,
                    None,
                    &[],
                    None,
                    None,
                    &body,
                    None,
                )
            })?;
            n += 1;
        }
        Ok(n)
    }

    /// Atomically (re)write the digest file, importing stragglers first.
    pub fn write_digest(
        &mut self,
        path: &Path,
        now: DateTime<Utc>,
        agents_section: &str,
    ) -> Result<usize> {
        let imported = match std::fs::read_to_string(path) {
            Ok(existing) => self.import_stragglers(&existing, now)?,
            Err(_) => 0,
        };
        let content = self.render_digest(now, agents_section)?;
        let tmp = path.with_extension("md.tmp");
        std::fs::write(&tmp, content)?;
        std::fs::rename(&tmp, path)?;
        Ok(imported)
    }
}

// ---------------------------------------------------------------------------------------------
// CLI glue: what `bf claim|heartbeat|release|note|board` call from main.rs.

const BOARD_FILE: &str = "bf.sqlite";
const DIGEST_FILE: &str = "AGENT_CHAT.md";
const PROOF_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(15 * 60);
const PROOF_OUTPUT_CAP: usize = 1024 * 1024;
const BODY_MAX: usize = 2000;

/// `$BF_DATA_DIR`, else `$HOME/.bf`. The board and the generated digest live here.
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

/// Open the board, creating the data directory 0700 when it is missing.
fn open_board() -> Result<(Board, PathBuf)> {
    let dir = data_dir();
    if !dir.is_dir() {
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&dir)?;
    }
    let board = Board::open(&dir.join(BOARD_FILE))?;
    Ok((board, dir))
}

/// Regenerate `AGENT_CHAT.md`. The live-agents section stays empty until discovery is wired in.
fn regenerate(board: &mut Board, dir: &Path, now: DateTime<Utc>) -> Result<()> {
    board.write_digest(&dir.join(DIGEST_FILE), now, "")?;
    Ok(())
}

/// The caller for reads (`bf board --to me`): `BF_AGENT` if set, else the detected identity.
fn caller() -> Identity {
    let declared = std::env::var("BF_AGENT").ok();
    identity::current(declared.as_deref())
}

/// `"\n{n} note(s) addressed to you — bf board --to me"` when `agent` has unread notes.
fn inbox_trailer(board: &Board, agent: &str) -> Result<String> {
    let unread = board.inbox(agent, board.inbox_seen(agent)?)?.len();
    Ok(if unread > 0 {
        format!("\n{unread} note(s) addressed to you — bf board --to me")
    } else {
        String::new()
    })
}

/// `--repo` if given, else the git toplevel of the cwd, else the cwd itself (with a warning).
fn resolve_repo(repo: Option<&str>) -> Result<String> {
    let cwd = std::env::current_dir()?;
    let dir = match repo {
        Some(r) => cwd.join(r),
        None => match crate::gitutil::git(&cwd, &["rev-parse", "--show-toplevel"]) {
            Ok(top) if !top.is_empty() => PathBuf::from(top),
            _ => {
                eprintln!(
                    "bf: {} is not inside a git repository; claiming against the directory itself",
                    cwd.display()
                );
                cwd
            }
        },
    };
    let dir = std::fs::canonicalize(&dir).unwrap_or(dir);
    Ok(dir.to_string_lossy().into_owned())
}

fn read_capped(mut input: impl std::io::Read) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = input.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        let room = PROOF_OUTPUT_CAP.saturating_sub(out.len());
        out.extend_from_slice(&chunk[..n.min(room)]);
    }
    Ok(out)
}

#[cfg(unix)]
fn kill_group(pid: u32) {
    let _ = Command::new("/bin/kill")
        .args(["-KILL", "--", &format!("-{pid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

/// Run `sh -c cmd` in `dir` with no stdin; returns the exit description and stdout+stderr
/// (capped at 1 MiB). Killed after `PROOF_TIMEOUT`.
fn run_proof(cmd: &str, dir: &Path) -> Result<(String, Vec<u8>)> {
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", cmd])
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::Other("proof stdout not captured".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::Other("proof stderr not captured".into()))?;
    let out = std::thread::spawn(move || read_capped(stdout));
    let err = std::thread::spawn(move || read_capped(stderr));
    let start = std::time::Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > PROOF_TIMEOUT {
            timed_out = true;
            #[cfg(unix)]
            kill_group(child.id());
            let _ = child.kill();
            break child.wait()?;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    // Anything the proof left behind holding the pipes would block the readers below.
    #[cfg(unix)]
    kill_group(child.id());
    let mut output = out
        .join()
        .map_err(|_| Error::Other("proof stdout reader failed".into()))??;
    let errors = err
        .join()
        .map_err(|_| Error::Other("proof stderr reader failed".into()))??;
    output.extend_from_slice(&errors);
    output.truncate(PROOF_OUTPUT_CAP);
    let code = if timed_out {
        "timeout".to_string()
    } else {
        status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "signal".to_string())
    };
    Ok((code, output))
}

/// Cut `s` to at most `max` bytes on a char boundary, marking the cut with `…`.
fn clamp(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_owned();
    }
    let mut end = max.saturating_sub('…'.len_utf8());
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}

/// Run the proof command and describe the result: exit code, digest of its output, the working
/// tree's changed paths and HEAD.
fn proof_body(cmd: &str, repo: &Path) -> Result<String> {
    let (code, output) = run_proof(cmd, repo)?;
    let sha = crate::digest::sha256_hex(&output);
    let changed: Vec<String> =
        crate::gitutil::git(repo, &["status", "--porcelain", "--untracked-files=all"])
            .map(|s| {
                s.lines()
                    .filter_map(|l| l.get(3..))
                    .map(|p| p.rsplit(" -> ").next().unwrap_or(p).to_string())
                    .collect()
            })
            .unwrap_or_default();
    let head = crate::gitutil::git(repo, &["rev-parse", "HEAD"]).unwrap_or_else(|_| "-".into());
    let changed = if changed.is_empty() {
        "none".to_string()
    } else {
        changed.join(",")
    };
    let prefix = format!("proof: {cmd} → exit {code}; sha256 {sha}; changed: ");
    let suffix = format!("; head {head}");
    let room = BODY_MAX.saturating_sub(prefix.len() + suffix.len());
    Ok(format!("{prefix}{}{suffix}", clamp(&changed, room)))
}

fn inbox_line(e: &Entry) -> String {
    format!(
        "#{} {} from {}: {}",
        e.seq,
        e.ts,
        e.agent,
        e.body.replace('\n', " ")
    )
}

fn entry_line(e: &Entry) -> String {
    let cid = e
        .claim_id
        .as_deref()
        .map(|c| format!(" {c}"))
        .unwrap_or_default();
    let to = e
        .to_agent
        .as_deref()
        .map(|t| format!(" →{t}"))
        .unwrap_or_default();
    format!(
        "{} {} {} {}{cid}{to} {}",
        e.seq,
        e.ts,
        e.kind,
        e.agent,
        e.body.replace('\n', " ")
    )
}

pub fn cli_claim(
    paths: &[String],
    message: &str,
    repo: Option<&str>,
    ttl_minutes: i64,
    agent: Option<&str>,
) -> Result<String> {
    let repo = resolve_repo(repo)?;
    let (mut board, dir) = open_board()?;
    let who = identity::current(agent);
    let now = Utc::now();
    let outcome = board.claim(&who, &repo, paths, message, ttl_minutes, now)?;
    regenerate(&mut board, &dir, now)?;
    let line = match outcome {
        ClaimOutcome::Claimed(c) => format!(
            "{} claimed {} in {} until {}",
            c.id,
            c.paths.join(", "),
            c.repo,
            c.expires_at
        ),
        ClaimOutcome::AlreadyHeld(c) => {
            format!("{} already held (same paths) until {}", c.id, c.expires_at)
        }
    };
    Ok(format!("{line}{}", inbox_trailer(&board, &who.agent)?))
}

pub fn cli_heartbeat(claim_id: &str, message: &str, agent: Option<&str>) -> Result<String> {
    let (mut board, dir) = open_board()?;
    let who = identity::current(agent);
    let now = Utc::now();
    let c = board.heartbeat(&who, claim_id, message, None, now)?;
    regenerate(&mut board, &dir, now)?;
    Ok(format!(
        "{} extended until {}{}",
        c.id,
        c.expires_at,
        inbox_trailer(&board, &who.agent)?
    ))
}

pub fn cli_release(
    claim_id: &str,
    message: Option<&str>,
    proof: Option<&str>,
    agent: Option<&str>,
) -> Result<String> {
    let (mut board, dir) = open_board()?;
    let who = identity::current(agent);
    let claim = board
        .active_claims(Utc::now())?
        .into_iter()
        .find(|c| c.id == claim_id)
        .ok_or(Error::StaleVersion)?;
    let body = match (proof, message) {
        (Some(cmd), message) => {
            let proof = proof_body(cmd, Path::new(&claim.repo))?;
            match message {
                Some(m) => format!("{proof} — {m}"),
                None => proof,
            }
        }
        (None, Some(m)) => m.to_string(),
        (None, None) => {
            return Err(Error::InvalidContract(
                "release needs -m \"<proof>\" or --proof '<command>'".into(),
            ))
        }
    };
    let now = Utc::now();
    let released = board.release(&who, claim_id, &body, now)?;
    regenerate(&mut board, &dir, now)?;
    Ok(format!(
        "{} released — {body}{}",
        released.id,
        inbox_trailer(&board, &who.agent)?
    ))
}

pub fn cli_note(
    message: &str,
    to: Option<&str>,
    re: Option<&str>,
    pr: Option<&str>,
    agent: Option<&str>,
) -> Result<String> {
    if to == Some("me") {
        return Err(Error::InvalidContract(
            "--to me: address a note to a named agent (see bf board)".into(),
        ));
    }
    let (mut board, dir) = open_board()?;
    let who = identity::current(agent);
    let now = Utc::now();
    let seq = board.note(&who, message, to, re, pr, now)?;
    regenerate(&mut board, &dir, now)?;
    Ok(format!(
        "note #{seq} posted{}",
        inbox_trailer(&board, &who.agent)?
    ))
}

pub fn cli_board(all: bool, to: Option<&str>) -> Result<String> {
    let (mut board, dir) = open_board()?;
    match to {
        Some(target) => {
            let me = (target == "me").then(|| caller().agent);
            let agent = me.as_deref().unwrap_or(target);
            let notes = board.inbox(agent, 0)?;
            if me.is_some() {
                let last = board.last_seq()?;
                board.mark_inbox_seen(agent, last)?;
            }
            if notes.is_empty() {
                return Ok(format!("no notes addressed to {agent}\n"));
            }
            Ok(notes.iter().map(|e| inbox_line(e) + "\n").collect())
        }
        None if all => Ok(board.all()?.iter().map(|e| entry_line(e) + "\n").collect()),
        None => {
            regenerate(&mut board, &dir, Utc::now())?;
            Ok(std::fs::read_to_string(dir.join(DIGEST_FILE))?)
        }
    }
}

/// Parse a `--ttl` such as `30m`, `2h`, `90` (minutes).
pub fn parse_ttl(s: &str) -> Result<i64> {
    let s = s.trim();
    let (num, unit) = match s.chars().last() {
        Some('m') => (&s[..s.len() - 1], 1),
        Some('h') => (&s[..s.len() - 1], 60),
        Some(c) if c.is_ascii_digit() => (s, 1),
        _ => {
            return Err(Error::InvalidContract(format!(
                "ttl {s:?}: use e.g. 30m or 2h"
            )))
        }
    };
    let n: i64 = num
        .parse()
        .map_err(|_| Error::InvalidContract(format!("ttl {s:?}: not a number")))?;
    let minutes = n * unit;
    if !(1..=MAX_TTL_MINUTES).contains(&minutes) {
        return Err(Error::InvalidContract(format!(
            "ttl must be 1..={MAX_TTL_MINUTES} minutes"
        )));
    }
    Ok(minutes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn who(name: &str, provider: &str) -> Identity {
        Identity {
            agent: name.into(),
            provider: provider.into(),
            pid: Some(42),
        }
    }
    fn t(s: &str) -> DateTime<Utc> {
        parse_ts(s).unwrap()
    }
    fn open() -> (tempfile::TempDir, Board) {
        let d = tempfile::tempdir().unwrap();
        let b = Board::open(&d.path().join("bf.sqlite")).unwrap();
        (d, b)
    }
    #[test]
    fn paths_normalise_and_reject() {
        assert_eq!(normalize_path("src/").unwrap(), "src/");
        assert_eq!(normalize_path("src/**").unwrap(), "src/");
        assert_eq!(normalize_path("./src/api.rs").unwrap(), "src/api.rs");
        assert_eq!(normalize_path(".").unwrap(), ".");
        for bad in ["/abs", "a/../b", "", "a\\b", "src/*.rs"] {
            assert!(normalize_path(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn overlap_is_component_wise() {
        assert!(overlaps("src/", "src/api.rs"));
        assert!(overlaps("src/api.rs", "src/"));
        assert!(overlaps("src", "src/api.rs"));
        assert!(!overlaps("src", "srcx"));
        assert!(!overlaps("src/a.rs", "src/b.rs"));
        assert!(overlaps(".", "anything/at/all"));
    }
    #[test]
    fn claim_conflict_idempotent_release() {
        let (_d, mut b) = open();
        let now = t("2026-09-18T16:00:00Z");
        let a = who("codex-1", "codex");
        let c = who("claude-orch", "claude");
        let ClaimOutcome::Claimed(c1) = b
            .claim(&a, "/r", &["src/".into()], "rebase", 30, now)
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(c1.id, "c-1");
        assert_eq!(c1.expires_at, "2026-09-18T16:30:00Z");
        let err = b
            .claim(&c, "/r", &["src/api.rs".into()], "trim", 30, now)
            .unwrap_err();
        assert!(matches!(err, Error::ResourceConflict(_)), "{err}");
        assert!(err.to_string().contains("codex-1") && err.to_string().contains("c-1"));
        // a different repo does not conflict
        assert!(matches!(
            b.claim(&c, "/other", &["src/api.rs".into()], "x", 30, now)
                .unwrap(),
            ClaimOutcome::Claimed(_)
        ));
        // identical re-claim by the same agent is idempotent
        assert!(matches!(
            b.claim(&a, "/r", &["src/**".into()], "again", 30, now).unwrap(),
            ClaimOutcome::AlreadyHeld(ref h) if h.id == "c-1"
        ));
        // release with proof, then the other agent may claim
        b.release(&a, "c-1", "cargo test exit 0", now).unwrap();
        assert!(matches!(
            b.claim(&c, "/r", &["src/api.rs".into()], "trim", 30, now)
                .unwrap(),
            ClaimOutcome::Claimed(_)
        ));
        // releasing twice is stale
        assert!(matches!(
            b.release(&a, "c-1", "x", now),
            Err(Error::StaleVersion)
        ));
    }
    #[test]
    fn heartbeat_extends_and_expiry_is_stale() {
        let (_d, mut b) = open();
        let a = who("grok-1", "grok");
        let t0 = t("2026-09-18T16:00:00Z");
        b.claim(&a, "/r", &["tests/".into()], "fixtures", 10, t0)
            .unwrap();
        let hb = b
            .heartbeat(&a, "c-1", "still going", None, t("2026-09-18T16:08:00Z"))
            .unwrap();
        assert_eq!(hb.expires_at, "2026-09-18T16:18:00Z");
        assert_eq!(b.active_claims(t("2026-09-18T16:17:59Z")).unwrap().len(), 1);
        assert_eq!(b.active_claims(t("2026-09-18T16:18:00Z")).unwrap().len(), 0);
        assert!(matches!(
            b.heartbeat(&a, "c-1", "late", None, t("2026-09-18T16:30:00Z")),
            Err(Error::StaleVersion)
        ));
        // after expiry another agent can take the paths
        assert!(matches!(
            b.claim(
                &who("cursor-1", "cursor"),
                "/r",
                &["tests/".into()],
                "mine",
                30,
                t("2026-09-18T16:30:00Z")
            )
            .unwrap(),
            ClaimOutcome::Claimed(_)
        ));
    }
    #[test]
    fn notes_inbox_and_status_cap() {
        let (_d, mut b) = open();
        let now = t("2026-09-18T16:00:00Z");
        let ben = Identity {
            agent: "ben".into(),
            provider: "human".into(),
            pid: None,
        };
        b.note(&ben, "please rebase", Some("codex-1"), None, None, now)
            .unwrap();
        b.note(
            &who("codex-1", "codex"),
            "ack",
            Some("ben"),
            None,
            None,
            now,
        )
        .unwrap();
        let long = "x".repeat(300);
        assert!(b.note(&ben, &long, None, None, None, now).is_err());
        assert!(b
            .note(
                &ben,
                &long,
                None,
                None,
                Some("https://github.com/x/y/pull/1"),
                now
            )
            .is_ok());
        let inbox = b.inbox("codex-1", 0).unwrap();
        assert_eq!(inbox.len(), 1);
        assert_eq!(inbox[0].body, "please rebase");
        assert!(b.inbox("codex-1", inbox[0].seq).unwrap().is_empty());
    }
    #[test]
    fn inbox_seen_marks_and_counts() {
        let (_d, mut b) = open();
        let now = t("2026-09-18T16:00:00Z");
        let ben = Identity {
            agent: "ben".into(),
            provider: "human".into(),
            pid: None,
        };
        assert_eq!(b.inbox_seen("codex-1").unwrap(), 0);
        assert_eq!(inbox_trailer(&b, "codex-1").unwrap(), "");
        b.note(&ben, "one", Some("codex-1"), None, None, now)
            .unwrap();
        b.note(&ben, "two", Some("codex-1"), None, None, now)
            .unwrap();
        assert_eq!(
            inbox_trailer(&b, "codex-1").unwrap(),
            "\n2 note(s) addressed to you — bf board --to me"
        );
        let last = b.last_seq().unwrap();
        b.mark_inbox_seen("codex-1", last).unwrap();
        assert_eq!(b.inbox_seen("codex-1").unwrap(), last);
        assert_eq!(inbox_trailer(&b, "codex-1").unwrap(), "");
        b.note(&ben, "three", Some("codex-1"), None, None, now)
            .unwrap();
        assert!(inbox_trailer(&b, "codex-1")
            .unwrap()
            .starts_with("\n1 note(s)"));
    }
    #[test]
    fn digest_round_trip_imports_stragglers() {
        let (d, mut b) = open();
        let now = t("2026-09-18T16:00:00Z");
        b.claim(
            &who("claude-orch", "claude"),
            "/r",
            &["src/board.rs".into()],
            "PR5",
            30,
            now,
        )
        .unwrap();
        let p = d.path().join("AGENT_CHAT.md");
        assert_eq!(
            b.write_digest(&p, now, "- claude 42 busy ~/bullet")
                .unwrap(),
            0
        );
        let s = std::fs::read_to_string(&p).unwrap();
        assert!(s.starts_with("# BulletFarm board — GENERATED"));
        assert!(s.contains("c-1") && s.contains("Live agents") && s.contains(END_MARKER));
        std::fs::write(&p, format!("{s}\n- 16:05Z someone — hand-written line\n")).unwrap();
        assert_eq!(
            b.write_digest(&p, t("2026-09-18T16:06:00Z"), "").unwrap(),
            1
        );
        let s2 = std::fs::read_to_string(&p).unwrap();
        assert!(s2.contains("legacy-md") && s2.contains("hand-written line"));
        assert!(s2.trim_end().ends_with(END_MARKER));
    }
    #[test]
    fn board_rows_are_immutable() {
        let (_d, mut b) = open();
        let now = t("2026-09-18T16:00:00Z");
        b.record(&who("x", "claude"), "stop", "stopped 1", now)
            .unwrap();
        assert!(b.conn.execute("UPDATE board SET body='y'", []).is_err());
        assert!(b.conn.execute("DELETE FROM board", []).is_err());
    }
    #[test]
    fn clamp_cuts_on_char_boundaries() {
        assert_eq!(clamp("abc", 3), "abc");
        assert_eq!(clamp("abcdef", 5), "ab…");
        assert_eq!(clamp("ééé", 5), "é…");
    }
    #[test]
    fn ttl_parses_minutes_and_hours() {
        assert_eq!(parse_ttl("30m").unwrap(), 30);
        assert_eq!(parse_ttl("2h").unwrap(), 120);
        assert_eq!(parse_ttl("45").unwrap(), 45);
        assert!(parse_ttl("0m").is_err());
        assert!(parse_ttl("4h").is_err());
        assert!(parse_ttl("soon").is_err());
    }
}
