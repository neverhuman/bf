use crate::error::{Error, Result};
use std::path::Path;
use std::process::Command;

pub fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|e| Error::Other(format!("git: {e}")))?;
    if !out.status.success() {
        return Err(Error::Other(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr)
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

pub fn init_repo(repo: &Path) -> Result<()> {
    std::fs::create_dir_all(repo)?;
    git(repo, &["init", "-b", "main"])?;
    git(repo, &["config", "user.email", "fake@bf.local"])?;
    git(repo, &["config", "user.name", "BulletFarm fake"])?;
    git(repo, &["config", "commit.gpgsign", "false"])?;
    Ok(())
}

pub fn commit_all(repo: &Path, message: &str) -> Result<(String, String)> {
    git(repo, &["add", "-A"])?;
    git(repo, &["commit", "-m", message])?;
    let commit = git(repo, &["rev-parse", "HEAD"])?;
    let tree = git(repo, &["rev-parse", "HEAD^{tree}"])?;
    Ok((commit, tree))
}
