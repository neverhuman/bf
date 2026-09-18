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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rev_parse_this_crate() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            git(root, &["rev-parse", "--is-inside-work-tree"]).unwrap(),
            "true"
        );
    }
}
