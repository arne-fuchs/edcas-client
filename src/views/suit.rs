use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{JournalData, SuitData, SuitWeapon};
use crate::views::ViewEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct SuitView {
    selected: usize,
    scroll: usize,
}

impl SuitView {
    pub fn new() -> Self {
        Self { selected: 0, scroll: 0 }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        let count = journal.pilot.suit.as_ref().map(|s| s.weapons.len()).unwrap_or(0);
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => {
                self.selected = self.selected.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('s') => {
                if count > 0 {
                    self.selected = (self.selected + 1).min(count - 1);
                }
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(10);
            }
            KeyCode::PageDown => {
                if count > 0 {
                    self.selected = (self.selected + 10).min(count - 1);
                }
            }
            _ => return ViewEvent::None,
        }
        ViewEvent::Consumed
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        self.render_list(frame, chunks[0], journal);
        self.render_detail(frame, chunks[1], journal);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        match &journal.pilot.suit {
            None => {
                lines.push(Line::from(Span::styled(
                    " No suit data — requires Odyssey and a SuitLoadout event",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            Some(suit) => {
                let visible = area.height.saturating_sub(2) as usize;
                let count = suit.weapons.len();

                if self.selected < self.scroll {
                    self.scroll = self.selected;
                } else if visible > 0 && self.selected >= self.scroll + visible {
                    self.scroll = self.selected + 1 - visible;
                }

                for (i, w) in suit.weapons.iter().enumerate().skip(self.scroll).take(visible) {
                    let selected = i == self.selected;
                    let slot = format!("{:<18}", w.slot.clone());
                    let name = format!("{:<28}", w.name.clone());
                    let grade = format!("G{}", w.class);
                    let mods_str = if w.mods.is_empty() {
                        String::new()
                    } else {
                        format!("  {}", w.mods.join(", "))
                    };

                    if selected {
                        lines.push(Line::from(Span::styled(
                            format!(" {slot}  {name}  {grade}{mods_str}"),
                            Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        let grade_color = Color::Rgb(180, 100, 255);
                        lines.push(Line::from(vec![
                            Span::styled(format!(" {slot}  {name}  "), Style::default().fg(Color::White)),
                            Span::styled(grade, Style::default().fg(grade_color)),
                            Span::styled(mods_str, Style::default().fg(Color::DarkGray)),
                        ]));
                    }
                }

                if count == 0 {
                    lines.push(Line::from(Span::styled(
                        " No weapons in current suit loadout",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
            }
        }

        let title = journal.pilot.suit.as_ref()
            .map(|s| format!(" Weapons ({}) ", s.weapons.len()))
            .unwrap_or_else(|| " Weapons ".to_string());

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0))),
            ),
            area,
        );
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        match &journal.pilot.suit {
            None => {
                lines.push(Line::from(Span::styled(
                    "No suit data",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            Some(suit) => {
                suit_info_lines(suit, &mut lines);

                if let Some(weapon) = suit.weapons.get(self.selected) {
                    weapon_detail_lines(weapon, &mut lines);
                }
            }
        }

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(" Detail ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            ),
            area,
        );
    }
}

fn suit_info_lines(suit: &SuitData, lines: &mut Vec<Line<'static>>) {
    let orange = Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
    let cyan   = Style::default().fg(Color::Cyan);
    let white  = Style::default().fg(Color::White);
    let purple = Style::default().fg(Color::Rgb(180, 100, 255));
    let dim    = Style::default().fg(Color::DarkGray);

    let grade_dots = "●".repeat(suit.grade as usize) + &"○".repeat(5usize.saturating_sub(suit.grade as usize));
    lines.push(Line::from(Span::styled(
        format!("{} G{}  {}", suit.suit_type, suit.grade, grade_dots),
        orange,
    )));

    if !suit.loadout_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Loadout  ", cyan),
            Span::styled(suit.loadout_name.clone(), white.add_modifier(Modifier::BOLD)),
        ]));
    }

    if !suit.mods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Suit Mods", cyan)));
        for m in &suit.mods {
            lines.push(Line::from(vec![
                Span::styled("  • ", dim),
                Span::styled(m.clone(), purple),
            ]));
        }
    }

    lines.push(Line::from(Span::styled("─".repeat(30), dim)));
}

fn weapon_detail_lines(w: &SuitWeapon, lines: &mut Vec<Line<'static>>) {
    let cyan   = Style::default().fg(Color::Cyan);
    let white  = Style::default().fg(Color::White);
    let purple = Style::default().fg(Color::Rgb(180, 100, 255));
    let dim    = Style::default().fg(Color::DarkGray);

    let grade_dots = "●".repeat(w.class as usize) + &"○".repeat(5usize.saturating_sub(w.class as usize));

    lines.push(Line::from(Span::styled(
        format!("{} G{}  {}", w.name, w.class, grade_dots),
        purple.add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(vec![
        Span::styled("Slot  ", cyan),
        Span::styled(w.slot.clone(), white),
    ]));

    if w.mods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("No engineering mods", dim)));
    } else {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Mods", cyan)));
        for m in &w.mods {
            lines.push(Line::from(vec![
                Span::styled("  • ", dim),
                Span::styled(m.clone(), purple),
            ]));
        }
    }
}
