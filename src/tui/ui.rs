//! Painting: the palette and the one `draw` that lays out
//! status | list + detail | footer, with overlays on top.

use super::model::{Model, Overlay};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

/// The four colours the console paints with.
///
/// Terminals that do not advertise 24-bit colour quantise RGB on their own
/// terms (navy → black, amber → muddy red), so pick a representation the
/// terminal can actually render instead of emitting truecolor and hoping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    pub bg: Color,
    pub text: Color,
    pub cyan: Color,
    pub amber: Color,
}

impl Palette {
    /// No colour at all: every cell inherits the terminal.
    pub fn none() -> Self {
        Self {
            bg: Color::Reset,
            text: Color::Reset,
            cyan: Color::Reset,
            amber: Color::Reset,
        }
    }
    /// Exact 24-bit colours, for terminals that advertise them.
    pub fn truecolor() -> Self {
        Self {
            bg: Color::Rgb(8, 16, 31),
            text: Color::Rgb(227, 237, 247),
            cyan: Color::Rgb(50, 220, 240),
            amber: Color::Rgb(255, 196, 77),
        }
    }
    /// Indexed ANSI, which every terminal renders exactly. The ground is left
    /// to the terminal so the console sits in the operator's own theme.
    pub fn indexed() -> Self {
        Self {
            bg: Color::Reset,
            text: Color::Reset,
            cyan: Color::Cyan,
            amber: Color::Yellow,
        }
    }
    /// `NO_COLOR` wins; otherwise trust `COLORTERM`, the only portable signal
    /// a terminal gives for 24-bit support.
    pub fn detect() -> Self {
        if std::env::var_os("NO_COLOR").is_some() {
            return Self::none();
        }
        match std::env::var("COLORTERM") {
            Ok(v) if v.contains("truecolor") || v.contains("24bit") => Self::truecolor(),
            _ => Self::indexed(),
        }
    }
}

const HELP: &str = "\
1 2 3        Agents · Board · PRs;  Tab / BackTab  next / previous screen
j k ↑ ↓      move; scroll the detail while it is focused
Enter        focus the detail;  Esc  back · clear filter · close overlay
/            type to filter rows (substring, case-insensitive); Enter keeps it
J            raw JSON in the detail;  t  follow the transcript tail (Agents)
x            Agents: stop the agent · Board: expire the claim (confirm y/N)
n            Agents: note to the agent · Board: unaddressed note
o            PRs: show the PR url;  r  refresh now
?            this help;  q / Ctrl+C  quit (nothing is killed)";

pub fn draw(frame: &mut Frame<'_>, model: &mut Model, palette: Palette) {
    let Palette {
        bg,
        text,
        cyan,
        amber,
    } = palette;
    let style = Style::default().fg(text).bg(bg);
    let area = frame.area();
    frame.render_widget(Block::default().style(style), area);
    let [status, body, footer] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(area);
    frame.render_widget(
        Paragraph::new(model.status_lines()).style(style.fg(amber)),
        status,
    );

    let split = [Constraint::Percentage(45), Constraint::Percentage(55)];
    let [list_area, detail_area] = if body.width >= 85 {
        Layout::horizontal(split).areas(body)
    } else {
        Layout::vertical(split).areas(body)
    };
    let items = model
        .rows
        .iter()
        .map(|r| ListItem::new(r.label.as_str()))
        .collect::<Vec<_>>();
    let position = model.selected().map_or(0, |i| i + 1);
    let title = format!("{} {position}/{}", model.view.title(), model.rows.len());
    let mut selection = ListState::default().with_selected(model.selected());
    let list = List::new(items)
        .block(Block::bordered().title(title))
        .style(style)
        .highlight_style(style.fg(cyan).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, list_area, &mut selection);

    let flags = [
        ("focused", model.details_focus),
        ("raw JSON", model.raw_json),
        ("tail", model.follow_tail),
    ];
    let flags = flags
        .iter()
        .filter(|(_, on)| *on)
        .map(|(name, _)| *name)
        .collect::<Vec<_>>();
    let title = if flags.is_empty() {
        "Detail".to_string()
    } else {
        format!("Detail [{}]", flags.join(" · "))
    };
    // Clamp the scroll to the wrapped line count so follow-tail (scroll = MAX)
    // lands on the last line instead of a blank pane.
    let inner_w = u32::from(detail_area.width.saturating_sub(2).max(1));
    let inner_h = u32::from(detail_area.height.saturating_sub(2));
    let lines: u32 = model
        .selected_detail()
        .lines()
        .map(|l| (l.chars().count() as u32).max(1).div_ceil(inner_w))
        .sum();
    model.scroll = model
        .scroll
        .min(lines.saturating_sub(inner_h).min(u32::from(u16::MAX)) as u16);
    frame.render_widget(
        Paragraph::new(model.selected_detail())
            .style(style)
            .wrap(Wrap { trim: false })
            .scroll((model.scroll, 0))
            .block(Block::bordered().title(title)),
        detail_area,
    );
    frame.render_widget(
        Paragraph::new(model.view.hint()).style(style.fg(cyan)),
        footer,
    );

    match &model.overlay {
        Overlay::None => {}
        Overlay::Help => {
            let help = Paragraph::new(HELP)
                .wrap(Wrap { trim: false })
                .block(Block::bordered().title("Help · any key closes"));
            popup(frame, area, 78, 11, help.style(style));
        }
        Overlay::Confirm { prompt, .. } => {
            let width = u16::try_from(prompt.chars().count() + 8).unwrap_or(u16::MAX);
            let confirm =
                Paragraph::new(format!("{prompt} y/N")).block(Block::bordered().title("Confirm"));
            popup(frame, area, width, 3, confirm.style(style.fg(amber)));
        }
        Overlay::Input { prompt, buffer, .. } => {
            let input = Paragraph::new(format!("{buffer}_"))
                .block(Block::bordered().title(prompt.as_str()));
            popup(frame, area, 64, 3, input.style(style));
        }
    }
}

fn popup(frame: &mut Frame<'_>, outer: Rect, width: u16, height: u16, widget: Paragraph<'_>) {
    let width = width.min(outer.width);
    let height = height.min(outer.height);
    let area = Rect::new(
        outer.x + (outer.width - width) / 2,
        outer.y + (outer.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);
    frame.render_widget(widget, area);
}
