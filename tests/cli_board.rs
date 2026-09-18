//! The board verbs end to end through the `bf` binary: claim → conflict → release with proof →
//! re-claim → stale release → addressed note → inbox → digest → legacy stragglers. The test process
//! is not under an agent, so `--as` names the writer (provider `human`).
use std::path::Path;
use std::process::{Command, Output};

struct Sandbox {
    data: tempfile::TempDir,
    repo: tempfile::TempDir,
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
    let data = tempfile::tempdir().unwrap();
    let repo = tempfile::tempdir().unwrap();
    git(repo.path(), &["init", "-q"]);
    std::fs::create_dir_all(repo.path().join("src")).unwrap();
    std::fs::write(repo.path().join("src/api.rs"), "// api\n").unwrap();
    git(repo.path(), &["add", "-A"]);
    git(repo.path(), &["commit", "-q", "-m", "base"]);
    Sandbox { data, repo }
}

impl Sandbox {
    fn bf(&self, args: &[&str]) -> Output {
        self.bf_as_env(args, None)
    }
    fn bf_as_env(&self, args: &[&str], bf_agent: Option<&str>) -> Output {
        let mut c = Command::new(env!("CARGO_BIN_EXE_bf"));
        c.args(args)
            .current_dir(self.repo.path())
            .env_remove("BF_AGENT")
            .env("BF_DATA_DIR", self.data.path())
            .env("HOME", self.data.path());
        if let Some(agent) = bf_agent {
            c.env("BF_AGENT", agent);
        }
        c.output().unwrap()
    }
    fn digest(&self) -> String {
        std::fs::read_to_string(self.data.path().join("AGENT_CHAT.md")).unwrap()
    }
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}
fn code(out: &Output) -> Option<i32> {
    out.status.code()
}

#[test]
fn claim_conflict_release_note_and_digest_round_trip() {
    let sb = sandbox();

    // 1. first claim
    let out = sb.bf(&["claim", "src/", "-m", "a", "--as", "codex-1"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with("c-1 claimed src/ in "),
        "{}",
        stdout(&out)
    );

    // 2. overlapping claim by another agent: exit 2 naming holder and claim id
    let out = sb.bf(&["claim", "src/api.rs", "-m", "b", "--as", "claude-orch"]);
    assert_eq!(code(&out), Some(2), "{}", stdout(&out));
    let err = stderr(&out);
    assert!(err.contains("codex-1") && err.contains("c-1"), "{err}");

    // 3. release with a proof command: it runs, exit code recorded
    let out = sb.bf(&["release", "c-1", "--proof", "true", "--as", "codex-1"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with("c-1 released — proof: true → exit 0;"),
        "{text}"
    );
    assert!(
        text.contains("sha256 ") && text.contains("changed: none") && text.contains("head "),
        "{text}"
    );

    // 4. now the second claim succeeds
    let out = sb.bf(&["claim", "src/api.rs", "-m", "b", "--as", "claude-orch"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let second_id = text.split_whitespace().next().unwrap().to_string();
    assert!(
        second_id.starts_with("c-") && text.contains(" claimed src/api.rs in "),
        "{text}"
    );

    // 5. releasing c-1 again is stale
    let out = sb.bf(&["release", "c-1", "-m", "x", "--as", "codex-1"]);
    assert_eq!(code(&out), Some(2), "{}", stdout(&out));
    assert!(stderr(&out).contains("STALE_VERSION"), "{}", stderr(&out));

    // 6. an addressed note
    let out = sb.bf(&[
        "note",
        "-m",
        "please rebase",
        "--to",
        "claude-orch",
        "--as",
        "codex-1",
    ]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).starts_with("note #"), "{}", stdout(&out));

    // 7. the recipient's inbox lists it
    let out = sb.bf(&["board", "--to", "claude-orch"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("from codex-1: please rebase"), "{text}");

    // 7b. the recipient's next write carries the inbox trailer, until they read `--to me`
    let out = sb.bf(&["heartbeat", &second_id, "--as", "claude-orch"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(&format!("{second_id} extended until ")),
        "{text}"
    );
    assert!(
        text.trim_end()
            .ends_with("1 note(s) addressed to you — bf board --to me"),
        "{text}"
    );
    let out = sb.bf_as_env(&["board", "--to", "me"], Some("claude-orch"));
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    assert!(stdout(&out).contains("please rebase"), "{}", stdout(&out));
    let out = sb.bf(&["heartbeat", &second_id, "--as", "claude-orch"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("note(s) addressed"),
        "{}",
        stdout(&out)
    );

    // 8. the digest
    let out = sb.bf(&["board"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.starts_with("# BulletFarm board — GENERATED"), "{text}");
    assert!(text.contains("c-"), "{text}");
    assert_eq!(text, sb.digest());

    // 9. a hand-written line after the end marker is imported as a legacy-md note
    let mut md = sb.digest();
    assert!(md.contains("<!-- bf:end -->"));
    md.push_str("\n- hand-written after the marker\n");
    std::fs::write(sb.data.path().join("AGENT_CHAT.md"), md).unwrap();
    let out = sb.bf(&["board"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("legacy-md") && text.contains("hand-written after the marker"),
        "{text}"
    );
    assert!(text.trim_end().ends_with("<!-- bf:end -->"), "{text}");
    let out = sb.bf(&["board", "--all"]);
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).contains(" note legacy-md - hand-written after the marker"),
        "{}",
        stdout(&out)
    );

    // 10. bad ttl is bad input
    let out = sb.bf(&["claim", "src/", "-m", "x", "--ttl", "9h"]);
    assert_eq!(code(&out), Some(64), "{}", stderr(&out));

    // `--to me` on a note is bad input
    let out = sb.bf(&["note", "-m", "hi", "--to", "me", "--as", "codex-1"]);
    assert_eq!(code(&out), Some(64), "{}", stderr(&out));
    assert!(sb.data.path().join("bf.sqlite").is_file());
}

#[test]
fn data_dir_is_created_private_under_home() {
    let sb = sandbox();
    let out = Command::new(env!("CARGO_BIN_EXE_bf"))
        .args(["board"])
        .current_dir(sb.repo.path())
        .env_remove("BF_AGENT")
        .env_remove("BF_DATA_DIR")
        .env("HOME", sb.data.path())
        .output()
        .unwrap();
    assert_eq!(code(&out), Some(0), "{}", stderr(&out));
    let dir = sb.data.path().join(".bf");
    assert!(dir.join("bf.sqlite").is_file() && dir.join("AGENT_CHAT.md").is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
}
