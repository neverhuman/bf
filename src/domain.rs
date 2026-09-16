use crate::error::{Error, Result};
use serde_json::Value;

pub fn require_schema_v3(value: &Value) -> Result<()> {
    match value.get("schema_version") {
        Some(Value::Number(n)) if n.as_u64() == Some(3) => Ok(()),
        other => Err(Error::InvalidContract(format!(
            "schema_version must be 3, got {other:?}"
        ))),
    }
}

pub fn require_id(value: &Value, field: &str) -> Result<String> {
    match value.get(field) {
        Some(Value::String(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Error::InvalidContract(format!("missing {field}"))),
    }
}

pub fn validate_write_path(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(Error::InvalidContract("empty path".into()));
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') || path.contains('\n') {
        return Err(Error::InvalidContract(format!("illegal path {path}")));
    }
    let stripped = path.strip_suffix('/').unwrap_or(path);
    for seg in stripped.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            return Err(Error::InvalidContract(format!(
                "illegal path segment in {path}"
            )));
        }
    }
    Ok(())
}

/// Segment-wise overlap, not raw string prefix.
pub fn paths_overlap(a: &str, b: &str) -> bool {
    let sa = segments(a);
    let sb = segments(b);
    sa.iter().zip(sb.iter()).all(|(x, y)| x == y)
}

fn segments(path: &str) -> Vec<&str> {
    let stripped = path.strip_suffix('/').unwrap_or(path);
    stripped.split('/').collect()
}

pub fn validate_task(value: &Value) -> Result<()> {
    require_schema_v3(value)?;
    let id = require_id(value, "id")?;
    if value.get("revision").and_then(Value::as_u64).unwrap_or(0) == 0 {
        return Err(Error::InvalidContract("revision must be > 0".into()));
    }
    require_id(value, "mission_id")?;
    require_id(value, "repo_id")?;
    require_id(value, "owner_id")?;
    let kind = require_id(value, "kind")?;
    if kind == "implementation" {
        let paths = value
            .get("write_paths")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::InvalidContract("code task needs write_paths".into()))?;
        if paths.is_empty() {
            return Err(Error::InvalidContract("code task needs write_paths".into()));
        }
        for p in paths {
            let s = p
                .as_str()
                .ok_or_else(|| Error::InvalidContract("write path must be a string".into()))?;
            validate_write_path(s)?;
        }
        match value.get("delivery_goal").and_then(Value::as_str) {
            Some("pr_ready" | "merged") => {}
            other => {
                return Err(Error::InvalidContract(format!(
                    "code task delivery_goal must be pr_ready or merged, got {other:?}"
                )))
            }
        }
    }
    let acceptance = value
        .get("acceptance")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::InvalidContract("acceptance required".into()))?;
    for item in acceptance {
        let checks = item.get("check_ids").and_then(Value::as_array);
        let manual = item.get("manual_owner_id").and_then(Value::as_str);
        let has_checks = checks.map(|c| !c.is_empty()).unwrap_or(false);
        if !has_checks && manual.is_none() {
            return Err(Error::CheckMissing(format!(
                "task {id} acceptance item has no check and no manual owner"
            )));
        }
    }
    if let Some(deps) = value.get("depends_on").and_then(Value::as_array) {
        for dep in deps {
            if dep
                .get("id")
                .or_else(|| dep.get("predecessor_id"))
                .is_none()
                && dep.as_str().is_none()
            {
                return Err(Error::UnknownDependency(format!(
                    "task {id} has a shapeless dependency"
                )));
            }
        }
    }
    Ok(())
}

pub fn cycle_in_ids(edges: &[(String, String)]) -> bool {
    use std::collections::{HashMap, HashSet};
    let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();
    for (a, b) in edges {
        graph.entry(a.as_str()).or_default().push(b.as_str());
        graph.entry(b.as_str()).or_default();
    }
    fn dfs<'a>(
        n: &'a str,
        graph: &HashMap<&'a str, Vec<&'a str>>,
        stack: &mut HashSet<&'a str>,
        seen: &mut HashSet<&'a str>,
    ) -> bool {
        if !stack.insert(n) {
            return true;
        }
        if seen.insert(n) {
            if let Some(next) = graph.get(n) {
                for c in next {
                    if dfs(c, graph, stack, seen) {
                        return true;
                    }
                }
            }
        }
        stack.remove(n);
        false
    }
    let mut seen = HashSet::new();
    let mut stack = HashSet::new();
    graph.keys().any(|n| dfs(n, &graph, &mut stack, &mut seen))
}

pub fn validate_profile(value: &Value) -> Result<()> {
    require_schema_v3(value)?;
    require_id(value, "id")?;
    if value.get("grant_id").is_some() {
        return Err(Error::InvalidContract("profile is not a grant".into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn path_rejects_dotdot() {
        assert!(validate_write_path("../x").is_err());
        assert!(validate_write_path("/abs").is_err());
        assert!(validate_write_path("src/dedup.py").is_ok());
        assert!(validate_write_path("src/").is_ok());
    }

    #[test]
    fn overlap_is_segment_wise() {
        assert!(paths_overlap("src/dedup.py", "src/dedup.py"));
        assert!(paths_overlap("src/", "src/dedup.py"));
        assert!(!paths_overlap("src", "src2/x"));
    }

    #[test]
    fn cycle_detect() {
        assert!(cycle_in_ids(&[
            ("A".into(), "B".into()),
            ("B".into(), "A".into())
        ]));
        assert!(!cycle_in_ids(&[("A".into(), "B".into())]));
    }

    #[test]
    fn task_needs_checks() {
        let mut t = json!({
            "schema_version": 3,
            "id": "T-x",
            "revision": 1,
            "mission_id": "M",
            "repo_id": "r",
            "owner_id": "o",
            "kind": "implementation",
            "write_paths": ["src/a.py"],
            "delivery_goal": "pr_ready",
            "acceptance": [{"id": "AC", "statement": "x", "check_ids": [], "manual_owner_id": null}]
        });
        assert!(validate_task(&t).is_err());
        t["acceptance"][0]["check_ids"] = json!(["c1"]);
        assert!(validate_task(&t).is_ok());
    }
}
