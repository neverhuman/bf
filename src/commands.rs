use crate::digest::{parse_strict_json, require_utf8, sha256_hex};
use crate::hub::{Hub, Operation};
use crate::{Error, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use uuid::Uuid;

const KINDS: &[&str] = &["note", "release", "stop"];

pub(crate) fn submit(hub: &Hub, actor: &str, raw: &[u8]) -> Result<Operation> {
    let text = require_utf8(raw)?;
    let body = parse_strict_json(text)?;
    let command_id = body["command_id"]
        .as_str()
        .ok_or_else(|| Error::InvalidContract("command_id required".into()))?
        .to_owned();
    if command_id.is_empty() || body.get("schema_version") != Some(&json!(3)) {
        return Err(Error::InvalidContract(
            "schema_version 3 and command_id required".into(),
        ));
    }
    let kind = body["kind"]
        .as_str()
        .ok_or_else(|| Error::InvalidContract("kind required".into()))?
        .to_owned();
    if !KINDS.contains(&kind.as_str()) {
        return Err(Error::InvalidContract(format!("unsupported kind {kind}")));
    }
    let digest = sha256_hex(raw);
    let actor = actor.to_owned();
    let raw_text = text.to_owned();
    hub.db.call(move |c| {
        let existing: Option<(String, String, String)> = c
            .query_row(
                "SELECT o.id, o.status, o.result_json FROM commands cmd
                 JOIN operations o ON o.command_id=cmd.command_id
                 WHERE cmd.command_id=?1",
                [&command_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .optional()?;
        if let Some((id, status, result_json)) = existing {
            let stored: String = c.query_row(
                "SELECT body_sha FROM commands WHERE command_id=?1",
                [&command_id],
                |r| r.get(0),
            )?;
            if stored != digest {
                return Err(Error::CommandConflict);
            }
            return Ok(Operation {
                id,
                status,
                result: serde_json::from_str(&result_json).unwrap_or(Value::Null),
            });
        }
        let op_id = format!("op-{}", Uuid::new_v4());
        let result = json!({
            "ok": false,
            "error": "NOT_IMPLEMENTED",
            "kind": kind,
            "note": "board/stop land in later PRs; this records the command for replay"
        });
        let now = Utc::now().to_rfc3339();
        c.execute(
            "INSERT INTO commands(command_id,actor_id,kind,body_sha,raw_json,created_at)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![command_id, actor, kind, digest, raw_text, now],
        )?;
        c.execute(
            "INSERT INTO operations(id,command_id,actor_id,status,result_json)
             VALUES(?1,?2,?3,'accepted',?4)",
            params![op_id, command_id, actor, result.to_string()],
        )?;
        Ok(Operation {
            id: op_id,
            status: "accepted".into(),
            result,
        })
    })
}

pub(crate) fn get(hub: &Hub, actor: &str, id: &str) -> Result<Operation> {
    let actor = actor.to_owned();
    let id = id.to_owned();
    hub.db.call(move |c| {
        c.query_row(
            "SELECT id,status,result_json FROM operations WHERE id=?1 AND actor_id=?2",
            params![id, actor],
            |r| {
                let raw: String = r.get(2)?;
                Ok(Operation {
                    id: r.get(0)?,
                    status: r.get(1)?,
                    result: serde_json::from_str(&raw).unwrap_or(Value::Null),
                })
            },
        )
        .optional()?
        .ok_or(Error::AuthRequired)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn replay_same_body_is_idempotent() {
        let dir = TempDir::new().unwrap();
        let hub = Hub::open(dir.path()).unwrap();
        let body =
            br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"hi"}}"#;
        let a = hub.command_bytes("owner", body).unwrap();
        let b = hub.command_bytes("owner", body).unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(a.status, "accepted");
    }

    #[test]
    fn changed_body_conflicts() {
        let dir = TempDir::new().unwrap();
        let hub = Hub::open(dir.path()).unwrap();
        hub.command_bytes(
            "owner",
            br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"a"}}"#,
        )
        .unwrap();
        let err = hub
            .command_bytes(
                "owner",
                br#"{"schema_version":3,"command_id":"c1","kind":"note","payload":{"text":"b"}}"#,
            )
            .unwrap_err();
        assert_eq!(err.code(), "COMMAND_CONFLICT");
    }
}
