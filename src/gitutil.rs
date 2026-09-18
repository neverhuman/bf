use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

pub fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let mut command = Command::new("git");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "commit.gpgsign=false",
            "-c",
            "protocol.ext.allow=never",
        ])
        .args(args)
        .current_dir(repo);
    let out = crate::runner::run(&mut command)?;
    if !out.status.success() {
        return Err(Error::Other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    String::from_utf8(out.stdout)
        .map(|s| s.trim_end().to_owned())
        .map_err(|_| Error::InvalidContract("Git output is not UTF-8".into()))
}
pub fn init_repo(repo: &Path) -> Result<()> {
    std::fs::create_dir_all(repo)?;
    git(repo, &["init", "-b", "main"])?;
    git(repo, &["config", "user.email", "fixture@bf.local"])?;
    git(repo, &["config", "user.name", "BulletFarm fixture"])?;
    Ok(())
}
pub fn commit_all(repo: &Path, message: &str) -> Result<(String, String)> {
    git(repo, &["add", "-A"])?;
    git(repo, &["commit", "-m", message])?;
    Ok((
        git(repo, &["rev-parse", "HEAD"])?,
        git(repo, &["rev-parse", "HEAD^{tree}"])?,
    ))
}
