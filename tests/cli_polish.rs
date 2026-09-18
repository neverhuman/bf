//! MVP polish through the `bf` binary: the screen's PR count and stale marker come from `prs`
//! (the fake `gh` in `tests/bin`, answering from `tests/fixtures/prs`), already on the first
//! frame. `plain()` prints no PR section, so the assertion reads the `BF_TUI_ONCE=1` status line.
//! `BF_HOME`/`BF_PROC` point at empty directories so the host's own sessions never leak in.
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Instant;

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

/// A committed git repository whose `origin` is `origin`.
fn repo_with_origin(origin: &str) -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    std::fs::create_dir_all(repo.path().join("src")).unwrap();
    std::fs::write(repo.path().join("src/lib.rs"), "// lib\n").unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-q", "-m", "base"]);
    git(repo.path(), &["remote", "add", "origin", origin]);
    repo
}

/// `PATH` with `bin` first, then the fake `gh` (`tests/bin`), then the real thing (for `git`).
fn shim_path(bin: Option<&Path>) -> String {
    let shim = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    let real = std::env::var("PATH").unwrap_or_default();
    match bin {
        Some(bin) => format!("{}:{}:{real}", bin.display(), shim.display()),
        None => format!("{}:{real}", shim.display()),
    }
}

/// Empty home, proc and data directories: no host sessions, a fresh board.
struct Farm {
    root: tempfile::TempDir,
}
impl Farm {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        for sub in ["home", "proc", "data"] {
            std::fs::create_dir_all(root.path().join(sub)).unwrap();
        }
        Self { root }
    }
    fn command(&self, cwd: &Path, path: &str) -> Command {
        // `setsid` detaches the controlling terminal so crossterm sizes the one-shot frame from
        // COLUMNS/LINES (through `tput`) instead of whatever terminal runs the tests.
        let mut command = Command::new("setsid");
        command
            .arg("-w")
            .arg(env!("CARGO_BIN_EXE_bf"))
            .current_dir(cwd)
            .env_remove("BF_AGENT")
            .env("HOME", self.root.path().join("home"))
            .env("BF_HOME", self.root.path().join("home"))
            .env("BF_PROC", self.root.path().join("proc"))
            .env("BF_DATA_DIR", self.root.path().join("data"))
            .env("PATH", path)
            .env("COLUMNS", "160")
            .env("LINES", "40")
            .env("TERM", "xterm")
            .stdin(Stdio::null());
        command
    }
    fn bf(&self, cwd: &Path, path: &str, args: &[&str]) -> Output {
        self.command(cwd, path).args(args).output().unwrap()
    }
    /// One `BF_TUI_ONCE=1` frame from `cwd`: (status line, whole frame, seconds it took).
    fn frame(&self, cwd: &Path, path: &str) -> (String, String, f64) {
        let start = Instant::now();
        let out = self
            .command(cwd, path)
            .env("BF_TUI_ONCE", "1")
            .output()
            .unwrap();
        let secs = start.elapsed().as_secs_f64();
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        let text = stdout(&out);
        let status = text.lines().next().unwrap_or_default().to_owned();
        (status, text, secs)
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn first_frame_counts_the_fixture_prs_and_marks_a_gh_outage() {
    let farm = Farm::new();
    let repo = repo_with_origin("git@github.com:neverhuman/fixture-a.git");
    let out = farm.bf(
        repo.path(),
        &shim_path(None),
        &["claim", "src/", "-m", "x", "--as", "t"],
    );
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));

    // The fake gh answers at once: the first frame already carries fixture-a's two PRs.
    let (status, text, secs) = farm.frame(repo.path(), &shim_path(None));
    assert!(status.starts_with("bf · agents 0"), "{text}");
    assert!(status.contains("claims 1"), "{text}");
    assert!(status.contains("PRs 2 open"), "{text}");
    assert!(!text.contains("gh unavailable"), "{text}");
    assert!(
        secs < 2.0,
        "the first frame took {secs:.2}s (loader budget is 2 s)"
    );

    // A gh that fails: no rows to keep in a fresh process, and the stale marker says so.
    let broken = tempfile::tempdir().unwrap();
    let gh = broken.path().join("gh");
    std::fs::write(&gh, "#!/bin/sh\necho 'gh: outage' >&2\nexit 1\n").unwrap();
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (status, text, _) = farm.frame(repo.path(), &shim_path(Some(broken.path())));
    assert!(status.contains("PRs 0 open"), "{text}");
    assert!(
        text.contains("gh unavailable: PRs stale since never"),
        "{text}"
    );
}
