use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub url: String,
    pub logical_key: String,
    pub head: String,
    pub base: String,
    pub title: String,
    pub human_closed: bool,
}
#[derive(Default, Serialize, Deserialize)]
struct State {
    next: u64,
    by_key: BTreeMap<String, PullRequest>,
}
/// A separate durable remote authority. No remote state is reconstructed from hub receipts.
pub struct FakeForge {
    path: PathBuf,
}
impl FakeForge {
    pub fn open(path: &Path) -> Result<Self> {
        fs::create_dir_all(path)?;
        Ok(Self {
            path: path.to_owned(),
        })
    }
    fn access<T>(&self, f: impl FnOnce(&mut State) -> Result<T>) -> Result<T> {
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(self.path.join("remote.lock"))?;
        lock.lock()?;
        let file = self.path.join("state.json");
        let mut state = match fs::read(&file) {
            Ok(bytes) => serde_json::from_slice(&bytes)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
            Err(e) => return Err(e.into()),
        };
        let result = f(&mut state)?;
        let temporary = self.path.join("state.pending");
        let mut out = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        out.write_all(&serde_json::to_vec(&state)?)?;
        out.sync_all()?;
        fs::rename(temporary, file)?;
        std::fs::File::open(&self.path)?.sync_all()?;
        Ok(result)
    }
    pub fn create(&self, key: &str, head: &str, base: &str, title: &str) -> Result<PullRequest> {
        self.access(|s| {
            if let Some(pr) = s.by_key.get(key) {
                if pr.human_closed || pr.head != head || pr.base != base {
                    return Err(Error::PolicyDenied(
                        "remote PR closed or edited by a human; reconciliation decision required"
                            .into(),
                    ));
                }
                return Ok(pr.clone());
            }
            s.next += 1;
            let pr = PullRequest {
                number: s.next,
                url: format!("fake://pr/{}", s.next),
                logical_key: key.into(),
                head: head.into(),
                base: base.into(),
                title: title.into(),
                human_closed: false,
            };
            s.by_key.insert(key.into(), pr.clone());
            Ok(pr)
        })
    }
    pub fn find(&self, key: &str) -> Result<Option<PullRequest>> {
        self.access(|s| Ok(s.by_key.get(key).cloned()))
    }
    pub fn mark_human_closed(&self, key: &str) -> Result<()> {
        self.access(|s| {
            if let Some(p) = s.by_key.get_mut(key) {
                p.human_closed = true;
            }
            Ok(())
        })
    }
}
