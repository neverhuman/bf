//! The operator screen. Bare `bf` opens it: a consumer of `Source` snapshots. Nothing here
//! mutates the farm except through `Source`, and only after an explicit confirm or typed note.
//! Quitting restores the terminal and kills nothing. Live data arrives in plan PR 7.

pub mod model;
pub mod text;
pub mod ui;

pub use model::plain;

use crate::Error;
use crossterm::event::{self, Event as TermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use model::{Action, Model, Overlay, View};
use serde::Serialize;
use std::io::{self, IsTerminal};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::{Duration, Instant};

/// What the screen reads and the few things it may do. Real data arrives in plan PR 7.
pub trait Source: Send + 'static {
    /// Called by the loader thread every 2 s, and on `r`.
    fn snapshot(&mut self) -> Snapshot;
    /// Agents `x`, after the operator confirms.
    fn stop(&mut self, pid: i64) -> std::result::Result<String, String>;
    /// Board `n` (unaddressed) or Agents `n` (to the selected agent).
    fn note(&mut self, to_agent: Option<String>, body: String) -> std::result::Result<(), String>;
    /// Board `x`, after the operator confirms.
    fn expire_claim(&mut self, claim_id: &str) -> std::result::Result<(), String>;
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Snapshot {
    pub at: String,
    pub agents: Vec<AgentRow>,
    pub claims: Vec<ClaimRow>,
    pub recent: Vec<EntryRow>,
    pub prs: Vec<PrRow>,
    pub prs_stale_since: Option<String>,
    pub load: Option<f64>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct AgentRow {
    pub provider: String,
    pub pid: i64,
    pub state: String,
    pub waiting_for: Option<String>,
    pub age_secs: Option<u64>,
    pub cwd: String,
    pub branch: Option<String>,
    pub title: Option<String>,
    pub last_prompt: Option<String>,
    pub detail: serde_json::Value,
    pub transcript_tail: Vec<String>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct ClaimRow {
    pub id: String,
    pub agent: String,
    pub provider: String,
    pub repo: String,
    pub paths: Vec<String>,
    pub expires_at: String,
    pub body: String,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct EntryRow {
    pub ts: String,
    pub kind: String,
    pub agent: String,
    pub claim_id: Option<String>,
    pub to_agent: Option<String>,
    pub body: String,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct PrRow {
    pub repo: String,
    pub number: u64,
    pub title: String,
    pub head_ref: String,
    /// CLEAN | DIRTY | BLOCKED | UNSTABLE | …
    pub state: String,
    pub draft: bool,
    pub checks_ok: u32,
    pub checks_fail: u32,
    pub checks_pending: u32,
    pub url: String,
    /// Exact missing merge preconditions.
    pub missing: Vec<String>,
}

/// A source with nothing behind it yet; lets the screen shell (PR 6) land before the data (PR 7).
pub struct StubSource;
impl Source for StubSource {
    fn snapshot(&mut self) -> Snapshot {
        Snapshot::default()
    }
    fn stop(&mut self, _pid: i64) -> std::result::Result<String, String> {
        Err("not implemented yet (plan PR 7)".into())
    }
    fn note(&mut self, _to: Option<String>, _body: String) -> std::result::Result<(), String> {
        Err("not implemented yet (plan PR 7)".into())
    }
    fn expire_claim(&mut self, _id: &str) -> std::result::Result<(), String> {
        Err("not implemented yet (plan PR 7)".into())
    }
}

enum Request {
    Refresh,
    Act(Action, String),
}
enum Reply {
    Snapshot(Box<Snapshot>),
    Done(std::result::Result<String, String>),
}

/// The loader thread owns the Source; the event loop only ever talks to it
/// through these channels, so a slow snapshot never holds the terminal.
fn start(mut source: Box<dyn Source>) -> (Sender<Request>, Receiver<Reply>) {
    let (requests, inbox) = mpsc::channel();
    let (replies, outbox) = mpsc::channel();
    std::thread::spawn(move || {
        while let Ok(request) = inbox.recv() {
            let reply = match request {
                Request::Refresh => Reply::Snapshot(Box::new(source.snapshot())),
                Request::Act(Action::Stop { pid }, _) => Reply::Done(source.stop(pid)),
                Request::Act(Action::Expire { claim_id }, _) => Reply::Done(
                    source
                        .expire_claim(&claim_id)
                        .map(|()| format!("expired {claim_id}")),
                ),
                Request::Act(Action::Note { to_agent }, body) => Reply::Done(
                    source
                        .note(to_agent, body)
                        .map(|()| "note posted".to_string()),
                ),
            };
            if replies.send(reply).is_err() {
                break;
            }
        }
    });
    (requests, outbox)
}

struct RestoreTerminal;
impl Drop for RestoreTerminal {
    fn drop(&mut self) {
        ratatui::restore();
    }
}

enum Command {
    None,
    Quit,
    Refresh,
    Send(Action, String),
}

fn key_command(model: &mut Model, key: KeyEvent) -> Command {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return Command::Quit;
    }
    if model.overlay != Overlay::None {
        return overlay_key(model, key);
    }
    if model.filtering {
        match key.code {
            KeyCode::Char(c) => model.filter_push(c),
            KeyCode::Backspace => model.filter_pop(),
            KeyCode::Enter => model.filtering = false,
            KeyCode::Esc => model.back(),
            _ => {}
        }
        return Command::None;
    }
    match key.code {
        KeyCode::Char('q') => return Command::Quit,
        KeyCode::Char('r') => return Command::Refresh,
        KeyCode::Char('?') => model.overlay = Overlay::Help,
        KeyCode::Char('1') => model.set_view(View::Agents),
        KeyCode::Char('2') => model.set_view(View::Board),
        KeyCode::Char('3') => model.set_view(View::Prs),
        KeyCode::Tab => model.set_view(model.view.shifted(1)),
        KeyCode::BackTab => model.set_view(model.view.shifted(-1)),
        KeyCode::Char('j') | KeyCode::Down => model.step(1),
        KeyCode::Char('k') | KeyCode::Up => model.step(-1),
        KeyCode::Enter => model.enter(),
        KeyCode::Esc => model.back(),
        KeyCode::Char('/') => model.start_filter(),
        KeyCode::Char('J') => model.raw_json = !model.raw_json,
        KeyCode::Char('t') => model.follow_tail = !model.follow_tail,
        KeyCode::Char('x') => model.confirm_action(),
        KeyCode::Char('n') => model.ask_note(),
        KeyCode::Char('o') => model.open_url(),
        _ => {}
    }
    Command::None
}

fn overlay_key(model: &mut Model, key: KeyEvent) -> Command {
    match &mut model.overlay {
        Overlay::Confirm { action, .. } => {
            let action = action.clone();
            model.overlay = Overlay::None;
            if matches!(key.code, KeyCode::Char('y' | 'Y')) {
                return Command::Send(action, String::new());
            }
        }
        Overlay::Input { action, buffer, .. } => match key.code {
            KeyCode::Char(c) => buffer.push(c),
            KeyCode::Backspace => {
                buffer.pop();
            }
            KeyCode::Esc => model.overlay = Overlay::None,
            KeyCode::Enter => {
                let (action, body) = (action.clone(), std::mem::take(buffer));
                model.overlay = Overlay::None;
                if !body.trim().is_empty() {
                    return Command::Send(action, body);
                }
            }
            _ => {}
        },
        _ => model.overlay = Overlay::None,
    }
    Command::None
}

/// One frame through a `TestBackend`, dumped to stdout. `BF_TUI_ONCE=1` for tests and CI.
fn once(model: &mut Model, width: u16, height: u16) -> io::Result<()> {
    let backend = ratatui::backend::TestBackend::new(width, height);
    let Ok(mut terminal) = ratatui::Terminal::new(backend);
    let Ok(_) = terminal.draw(|frame| ui::draw(frame, model, ui::Palette::none()));
    let buffer = terminal.backend().buffer();
    for line in buffer.content.chunks(usize::from(buffer.area.width.max(1))) {
        let text = line.iter().map(|cell| cell.symbol()).collect::<String>();
        println!("{}", text.trim_end());
    }
    Ok(())
}

fn io_error(e: io::Error) -> Error {
    Error::Other(e.to_string())
}

/// `BF_TUI_ONCE=1` dumps one frame; a non-tty stdout or `TERM=dumb` prints the plain snapshot;
/// otherwise the screen runs until `q`.
pub fn run(mut source: Box<dyn Source>) -> crate::Result<()> {
    if std::env::var("BF_TUI_ONCE").as_deref() == Ok("1") {
        let mut model = Model::default();
        model.update(source.snapshot());
        let (width, height) = crossterm::terminal::size().unwrap_or((80, 24));
        return once(&mut model, width, height).map_err(io_error);
    }
    if !io::stdout().is_terminal() || std::env::var("TERM").as_deref() == Ok("dumb") {
        print!("{}", plain(&source.snapshot()));
        return Ok(());
    }
    screen(source).map_err(io_error)
}

/// The interactive loop: paint, drain loader replies, poll keys, repeat.
fn screen(source: Box<dyn Source>) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let _restore = RestoreTerminal;
    let palette = ui::Palette::detect();
    let mut model = Model::default();
    // Paint before the first snapshot so a slow Source never shows a blank screen.
    terminal.draw(|frame| ui::draw(frame, &mut model, palette))?;
    let (requests, replies) = start(source);
    let send = |request| {
        requests
            .send(request)
            .map_err(|_| io::Error::other("loader thread stopped"))
    };
    send(Request::Refresh)?;
    let mut pending = true;
    let mut next_refresh = Instant::now() + Duration::from_secs(2);
    loop {
        while let Ok(reply) = replies.try_recv() {
            match reply {
                Reply::Snapshot(snapshot) => {
                    pending = false;
                    model.update(*snapshot);
                    next_refresh = Instant::now() + Duration::from_secs(2);
                }
                Reply::Done(result) => {
                    model.message = Some(result.unwrap_or_else(|e| format!("error: {e}")));
                    next_refresh = Instant::now();
                }
            }
        }
        if !pending && Instant::now() >= next_refresh {
            send(Request::Refresh)?;
            pending = true;
        }
        terminal.draw(|frame| ui::draw(frame, &mut model, palette))?;
        if !event::poll(Duration::from_millis(100))? {
            continue;
        }
        let TermEvent::Key(key) = event::read()? else {
            continue;
        };
        if key.kind == KeyEventKind::Release {
            continue;
        }
        match key_command(&mut model, key) {
            Command::Quit => break,
            Command::Refresh => next_refresh = Instant::now(),
            Command::Send(action, body) => send(Request::Act(action, body))?,
            Command::None => {}
        }
    }
    Ok(())
}
