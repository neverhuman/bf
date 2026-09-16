use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone)]
pub enum CreateOutcome {
    Confirmed(PullRequest),
    Lost,
}

#[derive(Debug, Default)]
pub struct FakeForge {
    next: u64,
    pub lose_next: bool,
    by_key: HashMap<String, PullRequest>,
}

impl FakeForge {
    pub fn create(
        &mut self,
        logical_key: &str,
        head: &str,
        base: &str,
        title: &str,
    ) -> CreateOutcome {
        if let Some(existing) = self.by_key.get(logical_key) {
            return CreateOutcome::Confirmed(existing.clone());
        }
        self.next += 1;
        let pr = PullRequest {
            number: self.next,
            url: format!("fake://pr/{}", self.next),
            logical_key: logical_key.to_owned(),
            head: head.to_owned(),
            base: base.to_owned(),
            title: title.to_owned(),
            human_closed: false,
        };
        self.by_key.insert(logical_key.to_owned(), pr.clone());
        if self.lose_next {
            self.lose_next = false;
            CreateOutcome::Lost
        } else {
            CreateOutcome::Confirmed(pr)
        }
    }

    pub fn find(&self, logical_key: &str) -> Option<&PullRequest> {
        self.by_key.get(logical_key)
    }

    pub fn mark_human_closed(&mut self, logical_key: &str) {
        if let Some(pr) = self.by_key.get_mut(logical_key) {
            pr.human_closed = true;
        }
    }
}
