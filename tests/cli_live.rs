use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;

fn write_proc(proc: &Path, pid: i64, exe: &str, cmdline: &str, comm: &str, cwd: &str) {
    let dir = proc.join(pid.to_string());
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("status"),
        format!("Name:\t{comm}\nState:\tS (sleeping)\nPPid:\t1\nVmRSS:\t  1024 kB\n"),
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
fn bare_bf_lists_fixture_sessions_and_claims() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let proc = root.path().join("proc");
    let data = root.path().join("data");
    let work = root.path().join("w");
    fs::create_dir_all(&home).unwrap();
    write_proc(
        &proc,
        42,
        "/usr/bin/claude",
        "claude",
        "claude",
        work.to_str().unwrap(),
    );

    let repo = root.path().join("repo");
    fs::create_dir_all(repo.join("src")).unwrap();
    Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap();
    fs::write(repo.join("src/a.rs"), "a\n").unwrap();
    Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t", "add", "-A"])
        .current_dir(&repo)
        .status()
        .unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "-m",
            "b",
        ])
        .current_dir(&repo)
        .status()
        .unwrap();
    let claim = Command::new(env!("CARGO_BIN_EXE_bf"))
        .args(["claim", "src/", "-m", "live", "--as", "codex-1"])
        .current_dir(&repo)
        .env("BF_DATA_DIR", &data)
        .env("HOME", &home)
        .output()
        .unwrap();
    assert!(
        claim.status.success(),
        "{}",
        String::from_utf8_lossy(&claim.stderr)
    );

    let out = Command::new(env!("CARGO_BIN_EXE_bf"))
        .env("BF_HOME", &home)
        .env("BF_PROC", &proc)
        .env("BF_DATA_DIR", &data)
        .env("HOME", &home)
        .env("TERM", "dumb")
        .args(["--data-dir", data.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("claude"), "{text}");
    assert!(
        text.contains("c-1") || text.contains("codex-1") || text.contains("src/"),
        "{text}"
    );
}
