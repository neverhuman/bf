//! Who is writing to the board: a declared name plus the *observed* provider and pid of the
//! nearest agent ancestor in the process tree. The declared name never hides the writer (spec §13).
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Identity {
    pub agent: String,
    pub provider: String,
    pub pid: Option<i64>,
}

/// Classify one process by its exe path, cmdline and comm. `None` when it is not an agent CLI.
pub fn classify(exe: &str, cmdline: &str, comm: &str) -> Option<&'static str> {
    if exe.contains("/.grok/") || comm == "grok" {
        return Some("grok");
    }
    if cmdline.contains("cursor-agent/versions/") || cmdline.contains("cursor-agent") {
        return Some("cursor");
    }
    if comm == "codex" || exe.ends_with("/codex") || cmdline.contains("@openai/codex") {
        return Some("codex");
    }
    if comm == "claude" || exe.ends_with("/claude") {
        return Some("claude");
    }
    None
}

fn read(proc_root: &Path, pid: i64, name: &str) -> String {
    std::fs::read(proc_root.join(pid.to_string()).join(name))
        .map(|b| {
            String::from_utf8_lossy(&b)
                .replace('\0', " ")
                .trim()
                .to_string()
        })
        .unwrap_or_default()
}

fn ppid(proc_root: &Path, pid: i64) -> Option<i64> {
    read(proc_root, pid, "status")
        .lines()
        .find_map(|l| l.strip_prefix("PPid:"))
        .and_then(|v| v.trim().parse::<i64>().ok())
        .filter(|p| *p > 0)
}

/// Walk the ancestor chain from `start` (inclusive) and return the first agent CLI found.
pub fn detect_ancestor(proc_root: &Path, start: i64) -> Option<(&'static str, i64)> {
    let mut pid = start;
    for _ in 0..64 {
        let exe = std::fs::read_link(proc_root.join(pid.to_string()).join("exe"))
            .map(|p: PathBuf| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let cmdline = read(proc_root, pid, "cmdline");
        let comm = read(proc_root, pid, "comm");
        if let Some(provider) = classify(&exe, &cmdline, &comm) {
            return Some((provider, pid));
        }
        pid = ppid(proc_root, pid)?;
        if pid <= 1 {
            return None;
        }
    }
    None
}

/// Resolve the writer identity. `declared` comes from `--as` or `BF_AGENT`.
pub fn resolve(declared: Option<&str>, proc_root: &Path, self_pid: i64) -> Identity {
    let observed = detect_ancestor(proc_root, self_pid);
    let (provider, pid) = match observed {
        Some((p, pid)) => (p.to_string(), Some(pid)),
        None => ("human".to_string(), None),
    };
    let agent = declared
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| match pid {
            Some(pid) => format!("{provider}-{pid}"),
            None => std::env::var("USER").unwrap_or_else(|_| "operator".into()),
        });
    Identity {
        agent,
        provider,
        pid,
    }
}

/// The identity of the current process, honouring `BF_PROC` (tests) for the proc root.
pub fn current(declared: Option<&str>) -> Identity {
    let proc_root = std::env::var_os("BF_PROC")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/proc"));
    resolve(declared, &proc_root, std::process::id() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fake_proc(entries: &[(i64, i64, &str, &str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (pid, ppid, exe, cmdline, comm) in entries {
            let p = d.path().join(pid.to_string());
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join("status"), format!("Name:\t{comm}\nPPid:\t{ppid}\n")).unwrap();
            std::fs::write(p.join("cmdline"), cmdline.replace(' ', "\0")).unwrap();
            std::fs::write(p.join("comm"), comm).unwrap();
            std::os::unix::fs::symlink(exe, p.join("exe")).unwrap();
        }
        d
    }
    #[test]
    fn classifies_each_provider() {
        assert_eq!(
            classify("/home/u/.grok/bin/agent", "agent --x", "agent"),
            Some("grok")
        );
        assert_eq!(
            classify("/usr/bin/node", "/home/u/.local/bin/agent --use-system-ca /home/u/.local/share/cursor-agent/versions/2026/index.js", "MainThread"),
            Some("cursor")
        );
        assert_eq!(classify("/x/codex", "codex --yolo", "codex"), Some("codex"));
        assert_eq!(
            classify("/home/u/.local/bin/claude", "claude --resume", "claude"),
            Some("claude")
        );
        assert_eq!(classify("/bin/bash", "bash", "bash"), None);
    }
    #[test]
    fn walks_to_the_nearest_agent_ancestor() {
        let d = fake_proc(&[
            (1, 0, "/sbin/init", "init", "init"),
            (
                100,
                1,
                "/home/u/.local/bin/claude",
                "claude --dangerously-skip-permissions",
                "claude",
            ),
            (200, 100, "/bin/bash", "bash -c bf claim", "bash"),
            (300, 200, "/home/u/.cargo/bin/bf", "bf claim src/", "bf"),
        ]);
        assert_eq!(detect_ancestor(d.path(), 300), Some(("claude", 100)));
        let id = resolve(None, d.path(), 300);
        assert_eq!(
            id,
            Identity {
                agent: "claude-100".into(),
                provider: "claude".into(),
                pid: Some(100)
            }
        );
        let named = resolve(Some("claude-orch"), d.path(), 300);
        assert_eq!(named.agent, "claude-orch");
        assert_eq!(named.pid, Some(100));
    }
    #[test]
    fn operator_shell_is_human() {
        let d = fake_proc(&[
            (1, 0, "/sbin/init", "init", "init"),
            (50, 1, "/bin/bash", "bash", "bash"),
        ]);
        let id = resolve(None, d.path(), 50);
        assert_eq!(id.provider, "human");
        assert_eq!(id.pid, None);
    }
}
