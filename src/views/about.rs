use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::event_shim::{KeyCode, KeyEvent};
use crate::views::ViewEvent;

pub struct AboutView {
    scroll: usize,
}

impl AboutView {
    pub fn new() -> Self {
        Self { scroll: 0 }
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up   => { self.scroll = self.scroll.saturating_sub(1); }
            KeyCode::Char('s') | KeyCode::Down => { self.scroll += 1; }
            KeyCode::PageUp   => { self.scroll = self.scroll.saturating_sub(10); }
            KeyCode::PageDown => { self.scroll += 10; }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = build_lines();
        let inner_height = area.height.saturating_sub(2) as usize;
        let max_scroll = lines.len().saturating_sub(inner_height);

        let paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" About ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .style(Style::default().fg(Color::White))
            .scroll((self.scroll.min(max_scroll) as u16, 0));

        frame.render_widget(paragraph, area);
    }
}

fn heading(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_owned(),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    ))
}

fn subheading(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))
}

fn ctrl(key: &str, desc: &str) -> Line<'static> {
    Line::from(vec![
        Span::styled(
            format!("    {:<20}", key),
            Style::default().fg(Color::Rgb(255, 140, 0)),
        ),
        Span::styled(desc.to_owned(), Style::default().fg(Color::White)),
    ])
}

fn note(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::DarkGray),
    ))
}

fn blank() -> Line<'static> {
    Line::from("")
}

include!("logo_lines.rs");

pub(super) fn build_lines() -> Vec<Line<'static>> {
    let mut lines = logo_lines();
    lines.push(blank());
    lines.extend(vec![
        Line::from(Span::styled(
            "EDCAS — Elite Dangerous Commander Assistant System",
            Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled("Version 0.4.0", Style::default().fg(Color::DarkGray))),
        blank(),
        Line::from(Span::styled(
            "A terminal-based assistant for Elite Dangerous commanders.",
            Style::default().fg(Color::White),
        )),
        Line::from(Span::styled(
            "Repository: https://github.com/arne-fuchs/edcas-client",
            Style::default().fg(Color::Cyan),
        )),
        blank(),

        // ── Global ───────────────────────────────────────────────
        heading("Global"),
        ctrl("q / e",              "Previous / next tab"),
        ctrl("x",                  "Quit"),
        blank(),

        // ── Navigation (shared by most views) ────────────────────
        heading("Navigation (most views)"),
        ctrl("w / ↑",             "Move cursor up"),
        ctrl("s / ↓",             "Move cursor down"),
        ctrl("d / →",             "Focus detail panel / next detail tab"),
        ctrl("a / ←",             "Focus list panel / previous detail tab"),
        blank(),

        // ── Stations & Carriers ───────────────────────────────────
        heading("Stations & Carriers"),
        ctrl("/ or f",             "Start a new search"),
        ctrl("Enter (typing)",     "Execute search"),
        ctrl("Esc (typing)",       "Cancel search"),
        ctrl("p",                  "Pin / unpin selected item"),
        ctrl("w / s",             "Move selection up / down in list"),
        ctrl("d / →",             "Open detail panel"),
        ctrl("a / ←",             "Back to list  /  prev detail tab"),
        subheading("Detail tabs: Overview · Market · Outfitting · Shipyard"),
        ctrl("d / →",             "Next detail tab"),
        ctrl("a / ←",             "Prev detail tab (or back to list)"),
        ctrl("w / s",             "Scroll detail content up / down"),
        blank(),

        // ── Factions ─────────────────────────────────────────────
        heading("Factions"),
        ctrl("/ or f",             "Start a new search"),
        ctrl("Enter (typing)",     "Execute search"),
        ctrl("Esc (typing)",       "Cancel search"),
        ctrl("p",                  "Pin / unpin selected faction"),
        ctrl("w / s",             "Move selection up / down in list"),
        ctrl("d / →",             "Open detail panel"),
        ctrl("a / ←",             "Back to list  /  prev detail tab"),
        subheading("Detail tabs: Info · Systems"),
        ctrl("d / →",             "Next detail tab"),
        ctrl("a / ←",             "Prev detail tab (or back to list)"),
        ctrl("w / s (Info)",      "Scroll info content up / down"),
        ctrl("w / s (Systems)",   "Select system row up / down"),
        ctrl("c (Systems)",       "Copy selected system name to clipboard"),
        blank(),

        // ── System ───────────────────────────────────────────────
        heading("System"),
        ctrl("w / s",             "Select faction in the faction list"),
        ctrl("Space",              "Open selected faction in the Factions tab"),
        blank(),

        // ── Explorer ─────────────────────────────────────────────
        heading("Explorer"),
        ctrl("w / s",             "Move up / down in the body tree"),
        ctrl("d / →",             "Expand node / open detail"),
        ctrl("a / ←",             "Collapse / go back"),
        blank(),

        // ── Settings ─────────────────────────────────────────────
        heading("Settings"),
        ctrl("w / s",             "Navigate rows"),
        ctrl("a",                  "Focus sidebar"),
        ctrl("d",                  "Focus fields"),
        ctrl("Space",              "Select or begin editing a field"),
        ctrl("Enter",              "Save edited field"),
        ctrl("Esc",                "Cancel edit"),
        blank(),

        // ── Inventory ────────────────────────────────────────────
        heading("Inventory (Materials / Ship Locker)"),
        ctrl("d / a  or  →/←",   "Switch between Materials and Ship Locker tabs"),
        note("Raw, Manufactured, Encoded shown in three columns with colour-coded progress bars."),
        note("Ship Locker shows Items, Components, Consumables, Data in four columns."),
        blank(),

        // ── General scrolling hint ────────────────────────────────
        heading("This screen"),
        ctrl("w / s  or  ↑/↓",   "Scroll up / down"),
    ]);
    lines
}
