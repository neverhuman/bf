use crate::hub::Hub;
use crate::{Error, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::process::Command;

pub const CODEX_VERSION: &str = "codex-cli 0.154.0";
/// Read-only local probes, never an execution grant, login, API-key fallback, or model call.
pub fn probe() -> Value {
    let version = crate::runner::run(Command::new("codex").arg("--version"))
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned());
    let authenticated = if version.is_some() {
        crate::runner::run(Command::new("codex").args(["login", "status"]))
            .ok()
            .map(|o| {
                let text = format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                if o.status.success() && text.to_lowercase().contains("chatgpt") {
                    "subscription_present"
                } else if o.status.success() {
                    "unsupported_auth_kind"
                } else {
                    "not_authenticated"
                }
            })
    } else {
        None
    };
    let linux = if cfg!(target_os = "linux") {
        crate::runner::run(Command::new("bwrap").args([
            "--unshare-all",
            "--die-with-parent",
            "--new-session",
            "--cap-drop",
            "ALL",
            "--ro-bind",
            "/usr",
            "/usr",
            "--ro-bind",
            "/lib",
            "/lib",
            "--ro-bind",
            "/lib64",
            "/lib64",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--tmpfs",
            "/tmp",
            "/usr/bin/true",
        ]))
        .map(|o| o.status.success())
        .unwrap_or(false)
    } else {
        false
    };
    json!({"observed_at":Utc::now().to_rfc3339(),"id":"codex","installed":version.is_some(),"version":version,"pinned_version":CODEX_VERSION,"version_matches":version.as_deref()==Some(CODEX_VERSION),"authenticated":authenticated.unwrap_or("unprobed"),"expired":"unknown (local login status is not a live authorization test)","qualified":false,"linux_namespace_probe":linux,"model_calls":0,"blockers":["Operating HOLD","No live execution grant","Linux filesystem, credential, descriptor, descendant, resource and network boundaries not certified","Codex JSONL transport and trusted live verification not certified","Real publisher and independent review not qualified"]})
}
impl Hub {
    pub fn probe_accounts(&self) -> Result<Value> {
        let report = probe();
        let raw = report.to_string();
        self.db.call(move|c|{c.execute("INSERT INTO observations VALUES(?1,'repo-demo','codex_qualification','local',?2,?3)",params![uuid::Uuid::new_v4().to_string(),Utc::now().to_rfc3339(),raw])?;Ok(())})?;
        Ok(report)
    }
    pub(crate) fn account_status(&self) -> Result<Value> {
        self.db.call(|c|{let raw:Option<String>=c.query_row("SELECT payload_json FROM observations WHERE source='codex_qualification' ORDER BY rowid DESC LIMIT 1",[],|r|r.get(0)).optional()?;Ok(raw.and_then(|s|serde_json::from_str(&s).ok()).unwrap_or(json!({"id":"codex","installed":"unprobed","authenticated":"unprobed","expired":"unknown","qualified":false})))})
    }
}
/// A fixture profile is never a shortcut to real execution, even when Codex is installed.
pub fn require_live_qualification(_profile: &Value) -> Result<()> {
    Err(Error::PolicyDenied("Live execution is unavailable: Operating HOLD, current live grant and independent boundary/transport certification are unresolved".into()))
}
#[cfg(test)]
mod tests {
    #[test]
    fn fixture_cannot_authorize_live() {
        assert!(super::require_live_qualification(
            &serde_json::json!({"fixture_only":true,"live_admission":true})
        )
        .is_err());
    }
}
