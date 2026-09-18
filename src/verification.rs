use crate::digest::sha256_hex;
use crate::hub::{BUGGY_PY, GOOD_PY, WRONG_PY};
use crate::{Error, Result};
use serde_json::{json, Value};
use std::path::Path;

const RECIPE: &str = include_str!("../fixtures/gates/dedup/check.py");
/// The fixture lane accepts only the embedded trusted gate corpus. Arbitrary candidate
/// execution needs a separately qualified Linux boundary and is refused before spawn.
pub(crate) fn verify_fixture(workspace: &Path) -> Result<Value> {
    let path = workspace.join("src/dedup.py");
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|_| Error::CheckMissing("missing required source".into()))?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(Error::CheckMissing("source must be a regular file".into()));
    }
    let bytes = std::fs::read(&path)?;
    if ![GOOD_PY.as_bytes(), BUGGY_PY.as_bytes(), WRONG_PY.as_bytes()].contains(&bytes.as_slice()) {
        return Err(Error::PolicyDenied("Untrusted source cannot execute in the fixture verifier; qualified isolation is required".into()));
    }
    let output = crate::runner::run(
        std::process::Command::new("python3")
            .args(["-I", "-S", "-c", RECIPE])
            .arg(workspace)
            .env_clear()
            .env("PATH", "/usr/bin:/bin"),
    )?;
    let receipt: Value = serde_json::from_slice(&output.stdout)
        .map_err(|_| Error::CheckMissing("missing complete protected result".into()))?;
    let required = ["first_delivery", "repeated_delivery", "distinct_delivery"];
    let tests = receipt["tests"]
        .as_array()
        .ok_or_else(|| Error::CheckMissing("required discovery missing".into()))?;
    if receipt["complete"] != true
        || tests.len() != required.len()
        || !required.iter().all(|name| {
            tests
                .iter()
                .filter(|t| {
                    t["name"] == *name
                        && t["executed"] == true
                        && t["assertions"] == 1
                        && t["skipped"] == false
                })
                .count()
                == 1
        })
    {
        return Err(Error::CheckMissing(
            "required tests/assertions/completion missing".into(),
        ));
    }
    let pass = output.status.success() && tests.iter().all(|t| t["passed"] == true);
    Ok(
        json!({"result":if pass{"pass"}else{"fail"},"source_digest":sha256_hex(&bytes),"recipe_digest":sha256_hex(RECIPE.as_bytes()),"required_tests":required,"execution":receipt,"fixture_only":true}),
    )
}
