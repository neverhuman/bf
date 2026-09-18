//! Screen-independent state: the data contract, snapshot → rows, selection,
//! filter and overlays. Nothing here touches a terminal or a Source.

use super::text::terminal_text;
use super::{AgentRow, ClaimRow, PrRow, Snapshot};
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum View {
    #[default]
    Agents,
    Board,
    Prs,
}
impl View {
    pub const ALL: [Self; 3] = [Self::Agents, Self::Board, Self::Prs];
    pub fn title(self) -> &'static str {
        match self {
            Self::Agents => "Agents",
            Self::Board => "Board",
            Self::Prs => "PRs",
        }
    }
    /// The screen `delta` tabs away, wrapping.
    pub fn shifted(self, delta: i32) -> Self {
        let i = Self::ALL.iter().position(|v| *v == self).unwrap_or(0) as i32;
        Self::ALL[(i + delta).rem_euclid(Self::ALL.len() as i32) as usize]
    }
    /// The one-line footer; the full keymap is under `?`.
    pub fn hint(self) -> &'static str {
        const HINTS: [&str; 3] = [
            "1-3 screens  j/k  / filter  x stop  n note  t tail  J raw  ? help  q quit",
            "1-3 screens  j/k  / filter  x expire  n note  J raw  ? help  q quit",
            "1-3 screens  j/k  / filter  o open  J raw  ? help  q quit",
        ];
        HINTS[self as usize]
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub label: String,
    pub human: String,
    pub raw: String,
}
fn row(value: &impl Serialize, id: String, label: String, human: String) -> Row {
    let raw = serde_json::to_string_pretty(value).unwrap_or_else(|_| "ENCODING_UNAVAILABLE".into());
    Row {
        id,
        label: terminal_text(&label).replace(['\n', '\t'], " "),
        human: terminal_text(&human),
        raw: terminal_text(&raw),
    }
}
fn field_lines(pairs: &[(&str, String)]) -> String {
    pairs
        .iter()
        .map(|(key, value)| format!("{key:<12} {value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// A mutation the operator asked for. It leaves the model as a loader request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Stop { pid: i64 },
    Expire { claim_id: String },
    Note { to_agent: Option<String> },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Overlay {
    #[default]
    None,
    Help,
    Confirm {
        prompt: String,
        action: Action,
    },
    Input {
        prompt: String,
        action: Action,
        buffer: String,
    },
}

#[derive(Default)]
pub struct Model {
    pub snapshot: Option<Snapshot>,
    pub view: View,
    pub rows: Vec<Row>,
    pub selected_id: Option<String>,
    pub filter: String,
    /// `/` was pressed: keys go into the filter until Enter or Esc.
    pub filtering: bool,
    pub details_focus: bool,
    pub scroll: u16,
    pub raw_json: bool,
    pub follow_tail: bool,
    pub overlay: Overlay,
    pub message: Option<String>,
}

impl Model {
    pub fn update(&mut self, snapshot: Snapshot) {
        self.snapshot = Some(snapshot);
        self.rebuild();
        if self.follow_tail && self.view == View::Agents {
            self.scroll = u16::MAX; // draw clamps this to the last line
        }
    }
    pub fn rebuild(&mut self) {
        let Some(snapshot) = &self.snapshot else {
            return;
        };
        let home = std::env::var("HOME").ok();
        let mut rows = match self.view {
            View::Agents => agent_rows(snapshot, home.as_deref()),
            View::Board => board_rows(snapshot, Utc::now()),
            View::Prs => pr_rows(snapshot),
        };
        let needle = self.filter.to_lowercase();
        if !needle.is_empty() {
            rows.retain(|r| r.label.to_lowercase().contains(&needle));
        }
        self.replace_rows(rows);
    }
    /// Swap in a new row set, keeping the selection by id when it survived.
    pub fn replace_rows(&mut self, rows: Vec<Row>) {
        self.rows = rows;
        if self.selected().is_none() {
            self.selected_id = self.rows.first().map(|r| r.id.clone());
            self.scroll = 0;
        }
    }
    pub fn selected(&self) -> Option<usize> {
        let id = self.selected_id.as_deref()?;
        self.rows.iter().position(|r| r.id == id)
    }
    pub fn set_view(&mut self, view: View) {
        if view != self.view {
            self.view = view;
            self.details_focus = false;
            self.scroll = 0;
            self.rebuild();
        }
    }
    pub fn step(&mut self, delta: i32) {
        if self.details_focus {
            self.scroll = (i32::from(self.scroll) + delta).clamp(0, i32::from(u16::MAX)) as u16;
        } else if !self.rows.is_empty() {
            let i =
                (self.selected().unwrap_or(0) as i32 + delta).rem_euclid(self.rows.len() as i32);
            self.selected_id = Some(self.rows[i as usize].id.clone());
            self.scroll = 0;
        }
    }
    pub fn enter(&mut self) {
        if self.selected().is_some() {
            self.details_focus = true;
        }
    }
    /// Esc: close the overlay, else clear the filter, else leave the detail, else clear the message.
    pub fn back(&mut self) {
        if self.overlay != Overlay::None {
            self.overlay = Overlay::None;
        } else if self.filtering || !self.filter.is_empty() {
            self.filter.clear();
            self.filtering = false;
            self.rebuild();
        } else if self.details_focus {
            self.details_focus = false;
        } else {
            self.message = None;
        }
    }
    pub fn start_filter(&mut self) {
        self.filtering = true;
        self.details_focus = false;
    }
    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.rebuild();
    }
    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.rebuild();
    }
    fn selected_agent(&self) -> Option<&AgentRow> {
        let id = self.selected_id.as_deref()?;
        self.snapshot
            .as_ref()?
            .agents
            .iter()
            .find(|a| agent_id(a) == id)
    }
    fn selected_claim(&self) -> Option<&ClaimRow> {
        let id = self.selected_id.as_deref()?;
        self.snapshot
            .as_ref()?
            .claims
            .iter()
            .find(|c| claim_id(c) == id)
    }
    fn selected_pr(&self) -> Option<&PrRow> {
        let id = self.selected_id.as_deref()?;
        self.snapshot.as_ref()?.prs.iter().find(|p| pr_id(p) == id)
    }
    /// `x`: stop the selected agent or expire the selected claim, after a confirm.
    pub fn confirm_action(&mut self) {
        let overlay = match self.view {
            View::Agents => self.selected_agent().map(|a| Overlay::Confirm {
                prompt: format!("stop {} {} in {}?", a.provider, a.pid, a.cwd),
                action: Action::Stop { pid: a.pid },
            }),
            View::Board => self.selected_claim().map(|c| Overlay::Confirm {
                prompt: format!("expire claim {}?", c.id),
                action: Action::Expire {
                    claim_id: c.id.clone(),
                },
            }),
            View::Prs => None,
        };
        if let Some(overlay) = overlay {
            self.overlay = overlay;
        }
    }
    /// `n`: a one-line note, to the selected agent on Agents and unaddressed on Board.
    pub fn ask_note(&mut self) {
        let to_agent = match self.view {
            View::Agents => match self.selected_agent() {
                Some(agent) => Some(agent_name(agent)),
                None => return,
            },
            View::Board => None,
            View::Prs => return,
        };
        let prompt = to_agent
            .as_deref()
            .map_or("note (unaddressed)".to_string(), |a| format!("note to {a}"));
        self.overlay = Overlay::Input {
            prompt,
            action: Action::Note { to_agent },
            buffer: String::new(),
        };
    }
    /// `o`: this draft only reports the url; nothing is spawned.
    pub fn open_url(&mut self) {
        if let Some(url) = self.selected_pr().map(|p| p.url.clone()) {
            self.message = Some(format!("open: {url}"));
        }
    }
    /// The two status-bar lines: counts, then filter / message / staleness.
    pub fn status_lines(&self) -> String {
        let Some(s) = &self.snapshot else {
            return format!("bf · waiting for the first snapshot\n{}", self.line2());
        };
        let mut counts: Vec<(&str, usize)> = ["claude", "codex", "grok", "cursor"]
            .map(|p| (p, 0))
            .to_vec();
        for a in &s.agents {
            match counts.iter_mut().find(|(p, _)| *p == a.provider) {
                Some(entry) => entry.1 += 1,
                None => counts.push((a.provider.as_str(), 1)),
            }
        }
        let providers = counts
            .iter()
            .map(|(p, n)| format!("{p} {n}"))
            .collect::<Vec<_>>()
            .join(" ");
        let busy = s.agents.iter().filter(|a| a.state == "busy").count();
        let load = s.load.map_or("-".to_string(), |l| format!("{l:.2}"));
        format!(
            "bf · agents {} ({providers}) · busy {busy} · claims {} · PRs {} open · snapshot {} · load {load}\n{}",
            s.agents.len(),
            s.claims.len(),
            s.prs.len(),
            clock(&s.at, "%H:%M:%S"),
            self.line2()
        )
    }
    fn line2(&self) -> String {
        if self.filtering || !self.filter.is_empty() {
            return format!(
                "filter: {}{}",
                self.filter,
                if self.filtering { "_" } else { "" }
            );
        }
        if let Some(message) = &self.message {
            return terminal_text(message);
        }
        match self
            .snapshot
            .as_ref()
            .and_then(|s| s.prs_stale_since.as_deref())
        {
            Some(since) => format!("gh unavailable: PRs stale since {}", terminal_text(since)),
            None => String::new(),
        }
    }
    pub fn selected_detail(&self) -> &str {
        match self.selected() {
            Some(i) if self.raw_json => &self.rows[i].raw,
            Some(i) => &self.rows[i].human,
            None if self.snapshot.is_none() => "Waiting for the first snapshot. q quits.",
            None => "No rows.",
        }
    }
}

fn agent_id(a: &AgentRow) -> String {
    format!("{}:{}", a.provider, a.pid)
}
/// The board identity bf gives an agent by default.
fn agent_name(a: &AgentRow) -> String {
    format!("{}-{}", a.provider, a.pid)
}
fn claim_id(c: &ClaimRow) -> String {
    format!("claim:{}", c.id)
}
fn pr_id(p: &PrRow) -> String {
    format!("{}#{}", p.repo, p.number)
}
fn agent_state(a: &AgentRow) -> String {
    match (&a.waiting_for, a.state.as_str()) {
        (Some(waiting), _) => format!("waiting: {waiting}"),
        (None, "") => "unknown".into(),
        (None, state) => state.into(),
    }
}
pub fn age(secs: Option<u64>) -> String {
    match secs {
        None => "-".into(),
        Some(s) if s < 60 => format!("{s}s"),
        Some(s) if s < 3600 => format!("{}m", s / 60),
        Some(s) if s < 86400 => format!("{}h", s / 3600),
        Some(s) => format!("{}d", s / 86400),
    }
}
fn until(expires_at: &str, now: DateTime<Utc>) -> String {
    let Ok(t) = DateTime::parse_from_rfc3339(expires_at) else {
        return truncate(expires_at, 12);
    };
    let left = (t.with_timezone(&Utc) - now).num_seconds();
    if left <= 0 {
        "expired".into()
    } else {
        age(Some(left.unsigned_abs()))
    }
}
fn clock(ts: &str, fmt: &str) -> String {
    DateTime::parse_from_rfc3339(ts)
        .map(|t| t.with_timezone(&Utc).format(fmt).to_string())
        .unwrap_or_else(|_| truncate(ts, 8))
}
fn short_cwd(cwd: &str, home: Option<&str>) -> String {
    let cwd = match home
        .filter(|h| !h.is_empty())
        .and_then(|h| cwd.strip_prefix(h))
    {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("~{rest}"),
        _ => cwd.to_string(),
    };
    let n = cwd.chars().count();
    if n <= 24 {
        cwd
    } else {
        format!("…{}", cwd.chars().skip(n - 23).collect::<String>())
    }
}
fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!(
            "{}…",
            s.chars().take(n.saturating_sub(1)).collect::<String>()
        )
    }
}

fn agent_rows(snapshot: &Snapshot, home: Option<&str>) -> Vec<Row> {
    let mut agents: Vec<&AgentRow> = snapshot.agents.iter().collect();
    agents.sort_by_key(|a| (a.state != "busy", a.age_secs.unwrap_or(u64::MAX)));
    agents
        .into_iter()
        .map(|a| {
            let state = agent_state(a);
            let branch = a.branch.as_deref().unwrap_or("-");
            let title = a.title.as_deref().unwrap_or("");
            let label = format!(
                "{} {} {} {} {} {} {}",
                a.provider,
                a.pid,
                state,
                age(a.age_secs),
                short_cwd(&a.cwd, home),
                branch,
                truncate(title, 40)
            );
            let mut human = field_lines(&[
                ("provider", a.provider.clone()),
                ("pid", a.pid.to_string()),
                ("state", state),
                ("cwd", a.cwd.clone()),
                ("branch", branch.to_string()),
                ("age", age(a.age_secs)),
                ("title", title.to_string()),
                ("last prompt", a.last_prompt.clone().unwrap_or_default()),
            ]);
            if !a.transcript_tail.is_empty() {
                human.push_str("\n\n");
                human.push_str(&a.transcript_tail.join("\n"));
            }
            row(&a.detail, agent_id(a), label, human)
        })
        .collect()
}
fn board_rows(snapshot: &Snapshot, now: DateTime<Utc>) -> Vec<Row> {
    let claims = snapshot.claims.iter().map(|c| {
        let paths = c.paths.join(",");
        let label = format!(
            "claim {} {} {} {} exp {} \"{}\"",
            c.id,
            c.agent,
            c.repo,
            paths,
            until(&c.expires_at, now),
            truncate(&c.body, 40)
        );
        let human = field_lines(&[
            ("claim", c.id.clone()),
            ("agent", c.agent.clone()),
            ("provider", c.provider.clone()),
            ("repo", c.repo.clone()),
            ("paths", paths.clone()),
            ("expires", c.expires_at.clone()),
            ("body", c.body.clone()),
        ]);
        row(c, claim_id(c), label, human)
    });
    let entries = snapshot.recent.iter().map(|e| {
        let mut label = format!("{} {} {}", clock(&e.ts, "%H:%MZ"), e.kind, e.agent);
        if let Some(claim) = &e.claim_id {
            label.push_str(&format!(" [{claim}]"));
        }
        if let Some(to) = &e.to_agent {
            label.push_str(&format!(" →{to}"));
        }
        label.push_str(&format!(" {}", truncate(&e.body, 60)));
        let human = field_lines(&[
            ("ts", e.ts.clone()),
            ("kind", e.kind.clone()),
            ("agent", e.agent.clone()),
            ("claim", e.claim_id.clone().unwrap_or_default()),
            ("to", e.to_agent.clone().unwrap_or_default()),
            ("body", e.body.clone()),
        ]);
        row(
            e,
            format!("entry:{}:{}:{}", e.ts, e.agent, e.kind),
            label,
            human,
        )
    });
    claims.chain(entries).collect()
}
fn pr_rows(snapshot: &Snapshot) -> Vec<Row> {
    snapshot
        .prs
        .iter()
        .map(|p| {
            let draft = if p.draft { " [draft]" } else { "" };
            let label = format!(
                "{}#{} {} checks:{}✓ {}✗ {}…{} {} ({})",
                p.repo,
                p.number,
                p.state,
                p.checks_ok,
                p.checks_fail,
                p.checks_pending,
                draft,
                truncate(&p.title, 40),
                p.head_ref
            );
            let mut human = field_lines(&[
                ("repo", p.repo.clone()),
                ("number", p.number.to_string()),
                ("title", p.title.clone()),
                ("head", p.head_ref.clone()),
                ("state", p.state.clone()),
                ("draft", p.draft.to_string()),
                (
                    "checks",
                    format!(
                        "{} ok, {} failed, {} pending",
                        p.checks_ok, p.checks_fail, p.checks_pending
                    ),
                ),
                ("url", p.url.clone()),
            ]);
            human.push_str("\n\nmissing:");
            if p.missing.is_empty() {
                human.push_str("\n  (none)");
            }
            for m in &p.missing {
                human.push_str(&format!("\n  - {m}"));
            }
            row(p, pr_id(p), label, human)
        })
        .collect()
}

/// Plain text for a non-tty stdout: agents, then claims, then recent entries.
pub fn plain(s: &Snapshot) -> String {
    let mut out = format!("bf snapshot {}\n\nAGENTS ({})\n", s.at, s.agents.len());
    for a in &s.agents {
        let title = a.title.as_deref().unwrap_or("");
        out.push_str(&format!(
            "  {:<7} {:<7} {:<12} {:<4} {}  {}\n",
            a.provider,
            a.pid,
            agent_state(a),
            age(a.age_secs),
            a.cwd,
            title
        ));
    }
    out.push_str(&format!("\nCLAIMS ({})\n", s.claims.len()));
    for c in &s.claims {
        out.push_str(&format!(
            "  {} {} {} {} exp {} — {}\n",
            c.id,
            c.agent,
            c.repo,
            c.paths.join(","),
            c.expires_at,
            c.body
        ));
    }
    out.push_str(&format!("\nRECENT ({})\n", s.recent.len()));
    for e in &s.recent {
        out.push_str(&format!("  {} {} {} {}\n", e.ts, e.kind, e.agent, e.body));
    }
    terminal_text(&out)
}
