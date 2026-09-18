//! The coordination board that replaces hand-edited AGENT_CHAT.md: append-only claims, heartbeats,
//! releases and addressed notes in SQLite. Implemented in plan PR 5.
use crate::{Error, Result};

pub const DEFAULT_TTL_MINUTES: i64 = 30;
pub const MAX_TTL_MINUTES: i64 = 180;

fn todo() -> Error {
    Error::Other("not implemented yet (plan PR 5: board.rs)".into())
}

pub fn cli_claim(
    _paths: &[String],
    _message: &str,
    _repo: Option<&str>,
    _ttl_minutes: i64,
    _agent: Option<&str>,
) -> Result<String> {
    Err(todo())
}
pub fn cli_heartbeat(_claim_id: &str, _message: &str, _agent: Option<&str>) -> Result<String> {
    Err(todo())
}
pub fn cli_release(
    _claim_id: &str,
    _message: Option<&str>,
    _proof: Option<&str>,
    _agent: Option<&str>,
) -> Result<String> {
    Err(todo())
}
pub fn cli_note(
    _message: &str,
    _to: Option<&str>,
    _re: Option<&str>,
    _pr: Option<&str>,
    _agent: Option<&str>,
) -> Result<String> {
    Err(todo())
}
pub fn cli_board(_all: bool, _to: Option<&str>) -> Result<String> {
    Err(todo())
}

/// Parse a `--ttl` such as `30m`, `2h`, `90` (minutes).
pub fn parse_ttl(s: &str) -> Result<i64> {
    let s = s.trim();
    let (num, unit) = match s.chars().last() {
        Some('m') => (&s[..s.len() - 1], 1),
        Some('h') => (&s[..s.len() - 1], 60),
        Some(c) if c.is_ascii_digit() => (s, 1),
        _ => {
            return Err(Error::InvalidContract(format!(
                "ttl {s:?}: use e.g. 30m or 2h"
            )))
        }
    };
    let n: i64 = num
        .parse()
        .map_err(|_| Error::InvalidContract(format!("ttl {s:?}: not a number")))?;
    let minutes = n * unit;
    if !(1..=MAX_TTL_MINUTES).contains(&minutes) {
        return Err(Error::InvalidContract(format!(
            "ttl must be 1..={MAX_TTL_MINUTES} minutes"
        )));
    }
    Ok(minutes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ttl_parses_minutes_and_hours() {
        assert_eq!(parse_ttl("30m").unwrap(), 30);
        assert_eq!(parse_ttl("2h").unwrap(), 120);
        assert_eq!(parse_ttl("45").unwrap(), 45);
        assert!(parse_ttl("0m").is_err());
        assert!(parse_ttl("4h").is_err());
        assert!(parse_ttl("soon").is_err());
    }
}
