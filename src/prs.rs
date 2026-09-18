//! Open pull requests across the repositories the board knows about, via `gh`. Implemented in
//! plan PR 8.
use crate::{Error, Result};
use serde::Serialize;

#[derive(Clone, Debug, Default, Serialize)]
pub struct Pr {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub head_ref: String,
    pub state: String,
    pub draft: bool,
    pub checks_ok: u32,
    pub checks_fail: u32,
    pub checks_pending: u32,
    pub url: String,
    /// Exact missing merge preconditions (CI, cross-vendor REVIEW comment, CLEAN).
    pub missing: Vec<String>,
}

pub fn list(_repos: &[String]) -> Result<Vec<Pr>> {
    Err(Error::Other(
        "not implemented yet (plan PR 8: prs.rs)".into(),
    ))
}
