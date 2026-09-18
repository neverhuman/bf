//! The TUI shell over a fake Source: rendering, model behaviour, sanitising.

use bf::tui::model::{plain, Model, Overlay, Row, View};
use bf::tui::text::terminal_text;
use bf::tui::ui::{draw, Palette};
use bf::tui::{AgentRow, ClaimRow, EntryRow, PrRow, Snapshot, Source};
use chrono::{TimeDelta, Utc};
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use ratatui::Terminal;
use serde_json::json;

#[derive(Default)]
struct FakeSource {
    stopped: Vec<i64>,
    notes: Vec<(Option<String>, String)>,
    expired: Vec<String>,
}
impl Source for FakeSource {
    fn snapshot(&mut self) -> Snapshot {
        fake_snapshot()
    }
    fn stop(&mut self, pid: i64) -> Result<String, String> {
        self.stopped.push(pid);
        Ok(format!("stopped {pid}"))
    }
    fn note(&mut self, to_agent: Option<String>, body: String) -> Result<(), String> {
        self.notes.push((to_agent, body));
        Ok(())
    }
    fn expire_claim(&mut self, claim_id: &str) -> Result<(), String> {
        if claim_id != "c-1" {
            return Err(format!("unknown claim {claim_id}"));
        }
        self.expired.push(claim_id.into());
        Ok(())
    }
}

fn agent(
    provider: &str,
    pid: i64,
    state: &str,
    waiting: Option<&str>,
    age: Option<u64>,
) -> AgentRow {
    AgentRow {
        provider: provider.into(),
        pid,
        state: state.into(),
        waiting_for: waiting.map(String::from),
        age_secs: age,
        cwd: format!("/home/ubuntu/work/{provider}"),
        branch: Some("main".into()),
        title: Some(format!("{provider} task")),
        last_prompt: Some("fix the tests".into()),
        detail: json!({ "provider": provider, "pid": pid }),
        transcript_tail: vec!["> cargo test".into(), "ok".into()],
    }
}
fn claim(id: &str, agent: &str, path: &str, minutes: i64) -> ClaimRow {
    ClaimRow {
        id: id.into(),
        agent: agent.into(),
        provider: agent.split('-').next().unwrap_or("").into(),
        repo: "bf".into(),
        paths: vec![path.into()],
        expires_at: (Utc::now() + TimeDelta::minutes(minutes)).to_rfc3339(),
        body: format!("working on {path}"),
    }
}
fn fake_snapshot() -> Snapshot {
    Snapshot {
        at: "2026-09-18T12:34:56Z".into(),
        agents: vec![
            agent("codex", 200, "waiting", Some("x"), Some(30)),
            agent("claude", 100, "busy", None, Some(120)),
            agent("grok", 300, "", None, None),
        ],
        claims: vec![
            claim("c-1", "claude-100", "src/tui", 22),
            claim("c-2", "codex-200", "src/board.rs", 5),
        ],
        recent: vec![
            EntryRow {
                ts: "2026-09-18T12:30:00Z".into(),
                kind: "note".into(),
                agent: "claude-100".into(),
                claim_id: Some("c-1".into()),
                to_agent: Some("codex-200".into()),
                body: "landing PR 6".into(),
            },
            EntryRow {
                ts: "2026-09-18T12:31:00Z".into(),
                kind: "claim".into(),
                agent: "codex-200".into(),
                claim_id: Some("c-2".into()),
                to_agent: None,
                body: "board.rs".into(),
            },
        ],
        prs: vec![PrRow {
            repo: "neverhuman/bf".into(),
            number: 6,
            title: "tui shell".into(),
            head_ref: "tui-shell".into(),
            state: "BLOCKED".into(),
            draft: true,
            checks_ok: 3,
            checks_fail: 1,
            checks_pending: 2,
            url: "https://github.com/neverhuman/bf/pull/6".into(),
            missing: vec!["review: 1 approval".into(), "checks: clippy".into()],
        }],
        prs_stale_since: Some("12:00:00".into()),
        load: Some(1.25),
    }
}
fn loaded() -> Model {
    let mut model = Model::default();
    model.update(FakeSource::default().snapshot());
    model
}
fn screen(terminal: &mut Terminal<TestBackend>, model: &mut Model) -> String {
    terminal
        .draw(|frame| draw(frame, model, Palette::none()))
        .unwrap();
    let buffer = terminal.backend().buffer();
    assert!(
        buffer
            .content
            .iter()
            .all(|c| c.fg == Color::Reset && c.bg == Color::Reset),
        "coloured cell under NO_COLOR"
    );
    buffer
        .content
        .chunks(usize::from(buffer.area.width))
        .map(|line| line.iter().map(|c| c.symbol()).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn renders_three_screens_monochrome_at_each_size() {
    for (width, height) in [(80, 24), (60, 30), (100, 30)] {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        let mut model = loaded();
        let text = screen(&mut terminal, &mut model);
        for needle in [
            "agents 3 (claude 1 codex 1 grok 1 cursor 0)",
            "gh unavailable: PRs stale since 12:00:00",
            "Agents 1/3",
            "waiting: x",
            "unknown",
            "1-3 screens  j/k  / filter  x stop",
        ] {
            assert!(
                text.contains(needle),
                "{width}x{height} missing {needle:?}:\n{text}"
            );
        }
        model.set_view(View::Board);
        let text = screen(&mut terminal, &mut model);
        assert!(
            text.contains("Board 1/4") && text.contains("claim c-1 claude-100 bf src/tui exp 2"),
            "{text}"
        );
        model.set_view(View::Prs);
        let text = screen(&mut terminal, &mut model);
        assert!(
            text.contains("PRs 1/1") && text.contains("neverhuman/bf#6 BLOCKED checks:3✓ 1✗ 2…"),
            "{text}"
        );
        assert_eq!(
            model.rows[0].label,
            "neverhuman/bf#6 BLOCKED checks:3✓ 1✗ 2… [draft] tui shell (tui-shell)"
        );
        assert!(text.contains("o open"), "{text}");
        model.enter();
        model.step(9); // past the end; draw clamps to the last wrapped line
        let text = screen(&mut terminal, &mut model);
        assert!(
            text.contains("Detail [focused]") && text.contains("- review: 1 approval"),
            "{text}"
        );
        assert!(model.scroll <= 2, "clamped, got {}", model.scroll);
        model.back();
        model.overlay = Overlay::Help;
        assert!(screen(&mut terminal, &mut model).contains("Help · any key closes"));
    }
}

#[test]
fn run_once_dumps_one_plain_frame() {
    std::env::set_var("BF_TUI_ONCE", "1");
    assert!(bf::tui::run(Box::new(FakeSource::default())).is_ok());
}

#[test]
fn step_enter_back() {
    let mut model = loaded();
    assert_eq!(model.selected(), Some(0));
    model.step(1);
    assert_eq!(model.selected_id.as_deref(), Some("codex:200"));
    model.step(-2);
    assert_eq!(model.selected_id.as_deref(), Some("grok:300"), "wraps");
    assert!(!model.details_focus);
    model.enter();
    assert!(model.details_focus);
    model.step(3);
    assert_eq!(
        (model.scroll, model.selected_id.as_deref()),
        (3, Some("grok:300")),
        "scrolls, keeps selection"
    );
    model.back();
    assert!(!model.details_focus);
    model.overlay = Overlay::Help;
    model.back();
    assert_eq!(model.overlay, Overlay::None);
    model.message = Some("open: x".into());
    model.back();
    assert_eq!(model.message, None);
}

#[test]
fn filter_narrows_rows_case_insensitively() {
    let mut model = loaded();
    model.start_filter();
    for c in "CODEX".chars() {
        model.filter_push(c);
    }
    assert_eq!(model.rows.len(), 1);
    assert!(model.rows[0].label.starts_with("codex 200 waiting: x 30s"));
    assert!(model.status_lines().ends_with("\nfilter: CODEX_"));
    model.filter_push('z');
    assert!(model.rows.is_empty() && model.selected().is_none());
    model.back();
    assert_eq!(
        (model.rows.len(), model.filter.as_str(), model.filtering),
        (3, "", false)
    );
}

#[test]
fn replace_rows_keeps_selection_by_id_across_snapshots() {
    let mut model = loaded();
    model.step(1);
    assert_eq!(model.selected_id.as_deref(), Some("codex:200"));
    let mut next = fake_snapshot();
    next.agents.remove(1); // claude leaves; codex is now first
    next.agents
        .push(agent("cursor", 400, "busy", None, Some(1)));
    model.update(next);
    assert_eq!(
        (model.selected_id.as_deref(), model.selected()),
        (Some("codex:200"), Some(1))
    );
    let make = |id: &str| Row {
        id: id.into(),
        label: id.into(),
        human: id.into(),
        raw: "{}".into(),
    };
    model.replace_rows(vec![make("a"), make("codex:200")]);
    assert_eq!(model.selected(), Some(1));
    model.replace_rows(vec![make("a"), make("b")]);
    assert_eq!(
        model.selected_id.as_deref(),
        Some("a"),
        "falls back to the first row"
    );
}

#[test]
fn agents_sort_busy_first_then_last_activity() {
    let model = loaded();
    let ids = model.rows.iter().map(|r| r.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, ["claude:100", "codex:200", "grok:300"]);
    assert!(model.rows[2].label.contains(" unknown - "));
}

#[test]
fn sanitizer_strips_escapes_controls_and_bidi() {
    assert_eq!(
        terminal_text("a\x1b[31mred\x1b[0m b\u{202e}c\x07d\n\te"),
        "ared bcd\n\te"
    );
    assert_eq!(
        terminal_text("x\x1b]52;clipboard\x07y\x1b(Bz\u{2066}\u{85}"),
        "xyz"
    );
}

#[test]
fn actions_confirm_note_and_fake_source_round_trip() {
    let mut model = loaded();
    model.confirm_action();
    assert!(
        matches!(&model.overlay, Overlay::Confirm { prompt, .. } if prompt == "stop claude 100 in /home/ubuntu/work/claude?")
    );
    model.back();
    model.ask_note();
    assert!(
        matches!(&model.overlay, Overlay::Input { prompt, .. } if prompt == "note to claude-100")
    );
    model.set_view(View::Board);
    model.confirm_action();
    assert!(
        matches!(&model.overlay, Overlay::Confirm { prompt, .. } if prompt == "expire claim c-1?")
    );
    model.set_view(View::Prs);
    model.open_url();
    assert_eq!(
        model.message.as_deref(),
        Some("open: https://github.com/neverhuman/bf/pull/6")
    );
    model.raw_json = true;
    assert!(model
        .selected_detail()
        .contains("\"head_ref\": \"tui-shell\""));

    let mut source = FakeSource::default();
    assert_eq!(source.stop(100), Ok("stopped 100".into()));
    assert_eq!(source.note(Some("claude-100".into()), "hi".into()), Ok(()));
    assert!(source.expire_claim("c-9").is_err() && source.expire_claim("c-1").is_ok());
    assert_eq!(
        (
            source.stopped.len(),
            source.notes.len(),
            source.expired.len()
        ),
        (1, 1, 1)
    );
    let text = plain(&source.snapshot());
    assert!(
        text.contains("AGENTS (3)") && text.contains("CLAIMS (2)") && text.contains("RECENT (2)")
    );
    assert!(text.contains("waiting: x") && text.contains("c-1 claude-100 bf src/tui exp "));
}
