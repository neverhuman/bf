//! Plan PR 7 wiring through the `bf` binary: bare `bf` shows the real agents and the real board,
//! and `bf stop <pid>` ends a process group and records it. `BF_HOME`/`BF_PROC` point at empty
//! directories so the host's own sessions never leak into these counts.
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

struct Sandbox {
    root: tempfile::TempDir,
}

fn git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn sandbox() -> Sandbox {
    let root = tempfile::tempdir().unwrap();
    for sub in ["home", "proc", "data", "repo/src"] {
        std::fs::create_dir_all(root.path().join(sub)).unwrap();
    }
    let repo = root.path().join("repo");
    git(&repo, &["init", "-q"]);
    std::fs::write(repo.join("src/api.rs"), "// api\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "base"]);
    Sandbox { root }
}

impl Sandbox {
    fn path(&self, sub: &str) -> std::path::PathBuf {
        self.root.path().join(sub)
    }
    fn bf(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_bf"))
            .args(args)
            .current_dir(self.path("repo"))
            .env_remove("BF_AGENT")
            .env("HOME", self.path("home"))
            .env("BF_HOME", self.path("home"))
            .env("BF_PROC", self.path("proc"))
            .env("BF_DATA_DIR", self.path("data"))
            .stdin(Stdio::null())
            .output()
            .unwrap()
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn bare_bf_shows_agents_and_the_board() {
    let sb = sandbox();
    let out = sb.bf(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("AGENTS (0)"), "{text}");
    assert!(text.contains("CLAIMS (0)"), "{text}");
    assert!(
        sb.path("data/AGENT_CHAT.md").is_file(),
        "the first snapshot writes the digest"
    );

    let out = sb.bf(&["claim", "src/", "-m", "x", "--as", "codex-1"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let out = sb.bf(&[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("CLAIMS (1)"), "{text}");
    assert!(text.contains("c-1 codex-1"), "{text}");
    assert!(text.contains("RECENT (1)"), "{text}");
}

#[test]
fn stop_refuses_pid_one() {
    let sb = sandbox();
    let out = sb.bf(&["stop", "1"]);
    assert_eq!(out.status.code(), Some(2), "{}", stdout(&out));
    assert!(stderr(&out).contains("POLICY_DENIED"), "{}", stderr(&out));
}

/// Kills the fixture process if an assertion fails before `bf stop` did.
struct Reap(Child);
impl Drop for Reap {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn stop_ends_a_process_group_and_records_it() {
    let sb = sandbox();
    let mut command = Command::new("sleep");
    command
        .arg("300")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    let mut child = Reap(command.spawn().unwrap());
    let pid = child.0.id().to_string();
    // This test may itself run under an agent (cargo test from a Claude Code shell), which would
    // trip the human-only guard. `BF_PROC` is a view of /proc holding only the target: bf finds
    // no agent ancestor of itself there, yet still reads the target's pgid and state.
    symlink(format!("/proc/{pid}"), sb.path("proc").join(&pid)).unwrap();

    let start = Instant::now();
    let out = sb.bf(&["stop", &pid]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(&format!("stopped {pid} (pgid {pid}) after TERM")),
        "{text}"
    );
    let mut exited = None;
    while exited.is_none() && start.elapsed() < Duration::from_secs(6) {
        exited = child.0.try_wait().unwrap();
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(exited.is_some(), "sleep {pid} still alive after 6 s");

    let out = sb.bf(&["board", "--all"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains(" stop ") && text.contains(&format!("stopped process {pid}")),
        "{text}"
    );
}
