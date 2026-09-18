//! The `bf` command surface exists end to end: every verb is listed, `doctor` works, and the
//! not-yet-implemented verbs fail with exit code 3 and say which plan PR delivers them.
use std::process::Command;

fn bf() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_bf"));
    c.env_remove("BF_AGENT");
    c
}

#[test]
fn help_lists_every_verb() {
    let out = bf().arg("--help").output().unwrap();
    assert!(out.status.success());
    let help = String::from_utf8_lossy(&out.stdout);
    for verb in [
        "agents",
        "board",
        "claim",
        "heartbeat",
        "release",
        "note",
        "stop",
        "prs",
        "run",
        "doctor",
        "serve",
        "web",
    ] {
        assert!(
            help.contains(&format!("\n  {verb}")),
            "missing verb {verb} in:\n{help}"
        );
    }
    assert!(!help.contains("\n  demo"), "demo must stay hidden");
}

#[test]
fn doctor_reports_sqlite_and_pins() {
    let dir = tempfile::tempdir().unwrap();
    let out = bf()
        .args(["--data-dir", dir.path().to_str().unwrap(), "doctor"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["sqlite"], "3.53.2");
    assert_eq!(v["toolchain"]["node_pin"], "v22.23.2");
    assert!(v["identity"]["provider"].is_string());
}

#[test]
fn unimplemented_verbs_exit_three_and_name_their_pr() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_str().unwrap();
    let args = ["run", "claude", "hello"];
    let out = bf().args(["--data-dir", d]).args(args).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(3),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("plan PR"),
        "{args:?}"
    );
}

#[test]
fn bad_input_exits_sixty_four() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path().to_str().unwrap();
    let out = bf()
        .args(["--data-dir", d, "claim", "src/", "-m", "x", "--ttl", "9h"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    let out = bf()
        .args(["--data-dir", d, "release", "c-1"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
    let out = bf()
        .args(["--data-dir", d, "run", "gemini", "hi"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(64));
}

#[test]
fn bare_bf_without_a_tty_prints_the_plain_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    // An empty `BF_PROC`/`BF_HOME`: the host's own agents must not leak into the count.
    let out = bf()
        .env("BF_DATA_DIR", dir.path())
        .env("BF_HOME", dir.path())
        .env("BF_PROC", dir.path())
        .args(["--data-dir", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("AGENTS ("),
        "expected an AGENTS header, got {stdout:?}"
    );
}
