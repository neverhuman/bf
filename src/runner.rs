use crate::{Error, Result};
use std::{
    cell::RefCell,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
thread_local! {static CANCEL:RefCell<Option<Arc<AtomicBool>>>=const {RefCell::new(None)};}
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn context(flag: Option<Arc<AtomicBool>>) {
    CANCEL.with(|c| *c.borrow_mut() = flag);
}
/// Supervisor for trusted fixture tools. This is process control, not a live OS sandbox.
pub(crate) fn run(command: &mut Command) -> Result<Output> {
    let flag = CANCEL.with(|c| c.borrow().clone());
    if flag.as_ref().is_some_and(|f| f.load(Ordering::Acquire)) {
        return Err(Error::PolicyDenied("stopped before spawn".into()));
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || bounded_read(stdout));
    let err = std::thread::spawn(move || bounded_read(stderr));
    let start = Instant::now();
    let mut cancelled = false;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if start.elapsed() > Duration::from_secs(30)
            || flag.as_ref().is_some_and(|f| f.load(Ordering::Acquire))
        {
            cancelled = true;
            #[cfg(unix)]
            {
                let _ = Command::new("/bin/kill")
                    .args(["-KILL", "--", &format!("-{}", child.id())])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = child.kill();
            break child.wait()?;
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Fixture tools do not daemonize. Kill any descendants holding inherited pipes on exit too.
    #[cfg(unix)]
    {
        let _ = Command::new("/bin/kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let stdout = out
        .join()
        .map_err(|_| Error::Other("stdout worker failed".into()))??;
    let stderr = err
        .join()
        .map_err(|_| Error::Other("stderr worker failed".into()))??;
    if cancelled {
        return Err(Error::PolicyDenied(
            "process terminated after stop/deadline".into(),
        ));
    }
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
fn bounded_read(mut input: impl Read) -> std::io::Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
        let n = input.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        if out.len() < 1024 * 1024 {
            out.extend_from_slice(&chunk[..n.min(1024 * 1024 - out.len())]);
        }
    }
    Ok(out)
}

// ---------------------------------------------------------------------------------------------
// Stopping a session: `bf stop <pid>` and the screen's `x`.

const TERM_GRACE: Duration = Duration::from_secs(5);
const KILL_GRACE: Duration = Duration::from_secs(2);

/// `/proc`, or `BF_PROC` for tests — the same override `identity` and `agents` honour.
fn proc_root() -> PathBuf {
    std::env::var_os("BF_PROC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/proc"))
}

/// Field `n` (1-based, as in proc(5)) of `<proc>/<pid>/stat`. The comm in field 2 may contain
/// spaces and parentheses, so the fields after it are counted from the last `)`.
pub(crate) fn stat_field(proc_root: &Path, pid: i64, n: usize) -> Option<String> {
    let stat = std::fs::read_to_string(proc_root.join(pid.to_string()).join("stat")).ok()?;
    let rest = &stat[stat.rfind(')')? + 1..];
    rest.split_whitespace()
        .nth(n.checked_sub(3)?)
        .map(str::to_owned)
}

fn ppid(proc_root: &Path, pid: i64) -> Option<i64> {
    std::fs::read_to_string(proc_root.join(pid.to_string()).join("status"))
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("PPid:"))
        .and_then(|v| v.trim().parse::<i64>().ok())
}

/// True when `target` is `start` itself or one of its ancestors (walking `PPid` from `start`).
pub fn in_ancestry(proc_root: &Path, start: i64, target: i64) -> bool {
    let mut pid = start;
    for _ in 0..64 {
        if pid == target {
            return true;
        }
        match ppid(proc_root, pid) {
            Some(p) if p > 1 => pid = p,
            _ => return false,
        }
    }
    false
}

/// Exited (no `/proc` entry) or a zombie its parent has not reaped yet.
fn gone(proc_root: &Path, pid: i64) -> bool {
    match std::fs::read_to_string(proc_root.join(pid.to_string()).join("status")) {
        Ok(status) => status.lines().any(|l| {
            l.strip_prefix("State:")
                .is_some_and(|v| v.trim().starts_with('Z'))
        }),
        Err(_) => true,
    }
}

fn wait_gone(proc_root: &Path, pid: i64, limit: Duration) -> bool {
    let start = Instant::now();
    loop {
        if gone(proc_root, pid) {
            return true;
        }
        if start.elapsed() >= limit {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// `kill -<sig> -- -<pgid>` through the kill binary (no libc crate). A failure only matters when
/// the target is still there, so the caller checks that.
fn signal_group(pgid: i64, sig: &str) -> std::io::Result<Output> {
    Command::new("/bin/kill")
        .args([&format!("-{sig}"), "--", &format!("-{pgid}")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
}

/// Stop the process group of `pid`: TERM, up to 5 s, then KILL, up to 2 s. Refuses pid ≤ 1, our
/// own session (the target is this process or one of its ancestors) and our own process group.
pub fn stop(pid: i64) -> Result<String> {
    stop_in(&proc_root(), i64::from(std::process::id()), pid)
}

fn stop_in(proc_root: &Path, me: i64, pid: i64) -> Result<String> {
    if pid <= 1 {
        return Err(Error::PolicyDenied(format!("refusing to stop pid {pid}")));
    }
    if in_ancestry(proc_root, me, pid) {
        return Err(Error::PolicyDenied(format!(
            "pid {pid} is this bf or one of its ancestors; stop it from another shell"
        )));
    }
    let pgid = stat_field(proc_root, pid, 5)
        .and_then(|s| s.parse::<i64>().ok())
        .ok_or_else(|| Error::Other(format!("no such process {pid}")))?;
    if pgid <= 1 {
        return Err(Error::PolicyDenied(format!(
            "pid {pid} is in process group {pgid}; refusing"
        )));
    }
    if stat_field(proc_root, me, 5).and_then(|s| s.parse::<i64>().ok()) == Some(pgid) {
        return Err(Error::PolicyDenied(format!(
            "pid {pid} shares this bf's process group {pgid}; stop it from another shell"
        )));
    }
    for (sig, grace) in [("TERM", TERM_GRACE), ("KILL", KILL_GRACE)] {
        let out = signal_group(pgid, sig)?;
        if !out.status.success() && !gone(proc_root, pid) {
            return Err(Error::Other(format!(
                "kill -{sig} -- -{pgid}: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )));
        }
        if wait_gone(proc_root, pid, grace) {
            return Ok(format!("stopped {pid} (pgid {pgid}) after {sig}"));
        }
    }
    Err(Error::OutcomeUnknown("still alive".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stop_observes_termination() {
        let flag = Arc::new(AtomicBool::new(false));
        context(Some(flag.clone()));
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            flag.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let result = run(Command::new("/bin/sh").args(["-c", "sleep 30 & wait"]));
        thread.join().unwrap();
        context(None);
        assert!(result.is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
    }

    /// (pid, ppid, pgid, comm)
    fn fake_proc(entries: &[(i64, i64, i64, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (pid, ppid, pgid, comm) in entries {
            let p = d.path().join(pid.to_string());
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(
                p.join("status"),
                format!("Name:\t{comm}\nState:\tS (sleeping)\nPPid:\t{ppid}\n"),
            )
            .unwrap();
            std::fs::write(
                p.join("stat"),
                format!("{pid} ({comm}) S {ppid} {pgid} {pgid} 0 -1 4194560 0 0 0 0 0 0 0 0 20 0 1 0 12345 0 0\n"),
            )
            .unwrap();
        }
        d
    }
    fn denied(r: Result<String>) -> String {
        match r {
            Err(Error::PolicyDenied(m)) => m,
            other => panic!("expected POLICY_DENIED, got {other:?}"),
        }
    }
    #[test]
    fn stat_fields_survive_spaces_and_parens_in_comm() {
        let d = fake_proc(&[(42, 7, 4242, "my (odd) comm")]);
        assert_eq!(stat_field(d.path(), 42, 3).as_deref(), Some("S"));
        assert_eq!(stat_field(d.path(), 42, 4).as_deref(), Some("7"));
        assert_eq!(stat_field(d.path(), 42, 5).as_deref(), Some("4242"));
        assert_eq!(stat_field(d.path(), 42, 22).as_deref(), Some("12345"));
        assert_eq!(stat_field(d.path(), 43, 5), None);
    }
    #[test]
    fn ancestry_guard_walks_ppid() {
        let d = fake_proc(&[
            (1, 0, 1, "init"),
            (100, 1, 100, "claude"),
            (200, 100, 100, "bash"),
            (300, 200, 100, "bf"),
            (555, 1, 555, "sleep"),
        ]);
        assert!(in_ancestry(d.path(), 300, 300));
        assert!(in_ancestry(d.path(), 300, 200));
        assert!(in_ancestry(d.path(), 300, 100));
        assert!(!in_ancestry(d.path(), 300, 555));
        assert!(!in_ancestry(d.path(), 300, 1));
        assert!(
            !in_ancestry(d.path(), 999, 100),
            "unknown start walks nowhere"
        );
    }
    #[test]
    fn stop_refuses_before_signalling() {
        let d = fake_proc(&[
            (1, 0, 1, "init"),
            (100, 1, 100, "claude"),
            (200, 100, 100, "bash"),
            (300, 200, 100, "bf"),
            (400, 1, 400, "codex"),
            (401, 400, 100, "child"),
            (500, 1, 1, "orphan"),
        ]);
        assert!(denied(stop(1)).contains("pid 1"));
        assert!(denied(stop(0)).contains("pid 0"));
        assert!(denied(stop_in(d.path(), 300, 100)).contains("ancestors"));
        assert!(denied(stop_in(d.path(), 300, 300)).contains("ancestors"));
        assert!(denied(stop_in(d.path(), 300, 401)).contains("process group 100"));
        assert!(denied(stop_in(d.path(), 300, 500)).contains("process group 1"));
        assert!(matches!(stop_in(d.path(), 300, 777), Err(Error::Other(m)) if m.contains("777")));
    }
}
