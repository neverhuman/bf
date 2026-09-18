use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use std::time::Instant;

fn bf() -> Command {
    Command::new(env!("CARGO_BIN_EXE_bf"))
}

fn write_proc(proc: &Path, pid: i64, ppid: i64, exe: &str, cmdline: &str, comm: &str, cwd: &str) {
    let dir = proc.join(pid.to_string());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("status"),
        format!("Name:\t{comm}\nState:\tS (sleeping)\nPPid:\t{ppid}\nVmRSS:\t  4096 kB\n"),
    )
    .unwrap();
    fs::write(dir.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
    fs::write(dir.join("comm"), comm).unwrap();
    let _ = fs::remove_file(dir.join("exe"));
    symlink(exe, dir.join("exe")).unwrap();
    fs::create_dir_all(cwd).unwrap();
    let _ = fs::remove_file(dir.join("cwd"));
    symlink(cwd, dir.join("cwd")).unwrap();
}

#[test]
fn lists_live_sessions_and_drops_dead_registry_rows() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let proc = root.path().join("proc");
    let work = root.path().join("work");
    fs::create_dir_all(home.join(".claude/sessions")).unwrap();
    fs::create_dir_all(home.join(".codex")).unwrap();
    fs::create_dir_all(home.join(".grok")).unwrap();

    write_proc(
        &proc,
        100,
        1,
        "/home/u/.local/bin/claude",
        "claude --resume abc",
        "claude",
        work.join("claude").to_str().unwrap(),
    );
    write_proc(
        &proc,
        200,
        1,
        "/usr/bin/codex",
        "codex --yolo",
        "codex",
        work.join("codex").to_str().unwrap(),
    );
    write_proc(
        &proc,
        300,
        1,
        "/usr/bin/node",
        "/home/u/.local/bin/agent --use-system-ca /home/u/.local/share/cursor-agent/versions/2026.09.10/index.js --resume=sess-cursor",
        "MainThread",
        work.join("cursor").to_str().unwrap(),
    );
    write_proc(
        &proc,
        400,
        1,
        "/home/u/.grok/downloads/grok-linux-x86_64",
        "grok",
        "agent",
        work.join("grok").to_str().unwrap(),
    );
    // Mention of cursor-agent in a shell argv must not become a Cursor row.
    write_proc(
        &proc,
        500,
        1,
        "/bin/bash",
        "bash -c pgrep -f cursor-agent/versions",
        "bash",
        work.join("bash").to_str().unwrap(),
    );

    fs::write(
        home.join(".claude/sessions/100.json"),
        r#"{"pid":100,"sessionId":"live-claude","cwd":"/work/claude","name":"fix tests","status":"busy","startedAt":1700000000000,"updatedAt":1700000001000}"#,
    )
    .unwrap();
    fs::write(
        home.join(".claude/sessions/999.json"),
        r#"{"pid":999,"sessionId":"dead-claude","cwd":"/gone","name":"stale","status":"idle"}"#,
    )
    .unwrap();
    fs::write(
        home.join(".codex/session_index.jsonl"),
        "{\"id\":\"codex-1\",\"thread_name\":\"Implement prune\",\"updated_at\":\"2026-09-18T00:00:00Z\"}\n",
    )
    .unwrap();
    fs::write(
        home.join(".grok/active_sessions.json"),
        r#"[{"session_id":"grok-1","pid":400,"cwd":"/work/grok","opened_at":"2026-09-18T00:00:00Z"}]"#,
    )
    .unwrap();

    let out = bf()
        .env("BF_HOME", &home)
        .env("BF_PROC", &proc)
        .args([
            "--data-dir",
            root.path().join("data").to_str().unwrap(),
            "agents",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let table = String::from_utf8_lossy(&out.stdout);
    assert!(table.contains("claude"), "{table}");
    assert!(table.contains("codex"), "{table}");
    assert!(table.contains("cursor"), "{table}");
    assert!(table.contains("grok"), "{table}");
    assert!(table.contains("fix tests"), "{table}");
    assert!(table.contains("Implement prune"), "{table}");
    assert!(!table.contains("dead-claude"), "{table}");
    assert!(!table.contains(" 500 "), "{table}");
}

#[test]
fn two_hundred_pids_under_500ms() {
    let root = tempfile::tempdir().unwrap();
    let proc = root.path().join("proc");
    let cwd = root.path().join("w");
    fs::create_dir_all(&cwd).unwrap();
    for pid in 1..=200 {
        write_proc(
            &proc,
            pid,
            1,
            "/usr/bin/codex",
            "codex --yolo",
            "codex",
            cwd.to_str().unwrap(),
        );
    }
    let start = Instant::now();
    let env = bf::agents::Env {
        home: root.path().join("empty-home"),
        proc,
    };
    let agents = bf::agents::discover(&env).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(agents.len(), 200);
    assert!(
        elapsed.as_millis() < 500,
        "discover 200 pids took {}ms",
        elapsed.as_millis()
    );
}
