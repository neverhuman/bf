//! `bf prs` end to end through the binary with a fake `gh` (`tests/bin/gh`) that answers from
//! `tests/fixtures/prs`, plus the pure pieces: origin URL → `owner/name` and the derivation of
//! `missing` from one `gh pr list --json` element.
use serde_json::json;
use std::path::Path;
use std::process::{Command, Output};

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

/// `PATH` with the fake `gh` first.
fn shim_path() -> String {
    let bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/bin");
    format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

fn bf(data: &Path, cwd: &Path, path: &str, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bf"))
        .args(args)
        .current_dir(cwd)
        .env_remove("BF_AGENT")
        .env("BF_DATA_DIR", data)
        .env("HOME", data)
        .env("PATH", path)
        .output()
        .unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}
fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn prs_lists_every_board_repo_through_gh() {
    let data = tempfile::tempdir().unwrap();
    let a = repo_with_origin("git@github.com:neverhuman/fixture-a.git");
    let b = repo_with_origin("https://github.com/neverhuman/fixture-b");
    for repo in [&a, &b] {
        let out = bf(
            data.path(),
            repo.path(),
            &shim_path(),
            &["claim", "src/", "-m", "x", "--as", "t"],
        );
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    }

    // From a directory that is not a repository: every slug comes from the board.
    let out = bf(data.path(), data.path(), &shim_path(), &["prs"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let text = stdout(&out);
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(
        lines.len(),
        3,
        "header + two PRs (fixture-b has none):\n{text}"
    );
    assert!(lines[0].starts_with("PR"), "{text}");

    let ready = lines
        .iter()
        .find(|l| l.starts_with("neverhuman/fixture-a#12"))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(ready.contains("CLEAN"), "{ready}");
    assert!(ready.contains("ok=2 fail=0 pending=0"), "{ready}");
    assert!(
        ready.contains("prs: open pull-request queue (feat/prs-queue)"),
        "{ready}"
    );
    assert!(ready.ends_with("-> ready"), "{ready}");
    assert!(!ready.contains("missing"), "{ready}");
    assert!(!ready.contains("draft"), "{ready}");

    let blocked = lines
        .iter()
        .find(|l| l.starts_with("neverhuman/fixture-a#13"))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(blocked.contains("DIRTY"), "{blocked}");
    assert!(blocked.contains("ok=0 fail=1 pending=1"), "{blocked}");
    assert!(
        blocked.contains("[draft] stop: SIGTERM then SIGKILL (wip/stop)"),
        "{blocked}"
    );
    for need in [
        "-> missing: ",
        "CI: 1 failing",
        "CI: 1 pending",
        "review: no REVIEW: approve 89abcdef0123456789abcdef0123456789abcdef comment",
        "mergeStateStatus: DIRTY",
        "; draft",
    ] {
        assert!(blocked.contains(need), "{need:?} not in {blocked}");
    }
}

#[test]
fn prs_uses_the_cwd_repo_when_the_board_is_empty() {
    let data = tempfile::tempdir().unwrap();
    let a = repo_with_origin("ssh://git@github.com/neverhuman/fixture-a.git");
    let out = bf(data.path(), a.path(), &shim_path(), &["prs"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stdout(&out).contains("neverhuman/fixture-a#12"),
        "{}",
        stdout(&out)
    );
    assert!(
        !data.path().join("bf.sqlite").exists(),
        "a read-only verb must not create the board"
    );
}

#[test]
fn prs_exits_one_when_gh_is_unavailable() {
    let data = tempfile::tempdir().unwrap();
    let a = repo_with_origin("github-neverhuman:neverhuman/fixture-a.git");
    let empty = tempfile::tempdir().unwrap();
    let out = bf(
        data.path(),
        a.path(),
        empty.path().to_str().unwrap(),
        &["prs"],
    );
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(stderr(&out).contains("gh unavailable"), "{}", stderr(&out));
    assert_eq!(stdout(&out), "");
}

#[test]
fn slug_from_origin_maps_every_github_form() {
    use bf::prs::slug_from_origin;
    for url in [
        "git@github.com:neverhuman/bf.git",
        "ssh://git@github.com/neverhuman/bf.git",
        "https://github.com/neverhuman/bf",
        "https://github.com/neverhuman/bf.git",
        "github-neverhuman:neverhuman/bf.git",
        "  https://github.com/neverhuman/bf/\n",
    ] {
        assert_eq!(
            slug_from_origin(url).as_deref(),
            Some("neverhuman/bf"),
            "{url}"
        );
    }
    for url in [
        "git@gitlab.com:neverhuman/bf.git",
        "https://example.com/neverhuman/bf",
        "https://github.com/neverhuman",
        "https://github.com/a/b/c",
        "/home/ubuntu/bullet/bf",
        "",
    ] {
        assert_eq!(slug_from_origin(url), None, "{url}");
    }
}

#[test]
fn missing_lists_each_unmet_merge_precondition() {
    use bf::prs::from_value;
    let sha = "0123456789abcdef0123456789abcdef01234567";
    let mut v = json!({
        "number": 7,
        "title": "t",
        "headRefName": "h",
        "headRefOid": sha,
        "isDraft": false,
        "mergeStateStatus": "CLEAN",
        "url": "https://github.com/o/n/pull/7",
        "statusCheckRollup": [{"conclusion": "SUCCESS"}, {"state": "SUCCESS"}],
        "reviews": [{"body": format!("looks good\nREVIEW: approve {sha}")}],
    });
    let pr = from_value("o/n", &v);
    assert_eq!(pr.repo, "o/n");
    assert_eq!(pr.number, 7);
    assert_eq!((pr.checks_ok, pr.checks_fail, pr.checks_pending), (2, 0, 0));
    assert_eq!(pr.state, "CLEAN");
    assert!(pr.missing.is_empty(), "{:?}", pr.missing);

    // A 7-character prefix approves; 6 does not; another sha does not.
    v["reviews"] = json!([{"body": format!("REVIEW: approve {}", &sha[..7])}]);
    assert!(from_value("o/n", &v).missing.is_empty());
    v["reviews"] = json!([{"body": format!("REVIEW: approve {}", &sha[..6])}]);
    assert_eq!(from_value("o/n", &v).missing.len(), 1);
    v["reviews"] = json!([{"body": "REVIEW: approve ffffffff"}, {"body": "REVIEW: changes"}]);
    assert_eq!(from_value("o/n", &v).missing.len(), 1);

    v["isDraft"] = json!(true);
    v["mergeStateStatus"] = json!("BLOCKED");
    v["statusCheckRollup"] = json!([
        {"conclusion": "FAILURE"},
        {"conclusion": "CANCELLED"},
        {"conclusion": null, "status": "QUEUED"},
        {"state": "PENDING"},
        {"conclusion": "SUCCESS"},
    ]);
    let pr = from_value("o/n", &v);
    assert!(pr.draft);
    assert_eq!((pr.checks_ok, pr.checks_fail, pr.checks_pending), (1, 2, 2));
    assert_eq!(
        pr.missing,
        vec![
            "CI: 2 failing".to_string(),
            "CI: 2 pending".to_string(),
            format!("review: no REVIEW: approve {sha} comment"),
            "mergeStateStatus: BLOCKED".to_string(),
            "draft".to_string(),
        ]
    );

    // Nothing at all: tolerated, and every precondition is missing.
    let pr = from_value("o/n", &json!({}));
    assert_eq!(pr.state, "UNKNOWN");
    assert_eq!(
        pr.missing,
        vec![
            "review: no REVIEW: approve <head-sha> comment".to_string(),
            "mergeStateStatus: UNKNOWN".to_string(),
        ]
    );
}
