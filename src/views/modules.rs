use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::views::ViewEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub struct ModulesView {
    selected: usize,
    scroll: usize,
}

impl ModulesView {
    pub fn new() -> Self {
        Self { selected: 0, scroll: 0 }
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        let count = journal.modules.len();
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
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);

        self.render_list(frame, chunks[0], journal);
        self.render_detail(frame, chunks[1], journal);
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let visible = area.height.saturating_sub(2) as usize;
        let count = journal.modules.len();

        if self.selected < self.scroll {
            self.scroll = self.selected;
        } else if visible > 0 && self.selected >= self.scroll + visible {
            self.scroll = self.selected + 1 - visible;
        }

        let mut lines: Vec<Line<'static>> = Vec::new();

        for (i, m) in journal.modules.iter().enumerate().skip(self.scroll).take(visible) {
            let selected = i == self.selected;
            let slot = format!("{:<30}", m.slot.clone());
            let item = format!("{:<40}", m.item.clone());
            let power = if m.power > 0.0 {
                format!("{:.2} MW", m.power)
            } else {
                "  —     ".to_string()
            };
            let style = if selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!(" {slot}  {item}  {power}"),
                style,
            )));
        }

        if journal.modules.is_empty() {
            lines.push(Line::from(Span::styled(
                " No module data — ensure journal directory is configured",
                Style::default().fg(Color::DarkGray),
            )));
        }

        frame.render_widget(
            Paragraph::new(lines).block(
                Block::default()
                    .title(format!(" Modules ({count}) "))
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Rgb(255, 140, 0))),
            ),
            area,
        );
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if let Some(m) = journal.modules.get(self.selected) {
            lines.push(Line::from(Span::styled(
                "Slot",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(Line::from(Span::styled(
                m.slot.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Item",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(Line::from(Span::styled(
                m.item.clone(),
                Style::default().fg(Color::White),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Power Draw",
                Style::default().fg(Color::Cyan),
            )));
            let pwr = if m.power > 0.0 {
                format!("{:.2} MW", m.power)
            } else {
                "—".to_string()
            };
            lines.push(Line::from(Span::styled(
                pwr,
                Style::default().fg(Color::Rgb(255, 140, 0)),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Priority",
                Style::default().fg(Color::Cyan),
            )));
            lines.push(Line::from(Span::styled(
                if m.power > 0.0 { m.priority.to_string() } else { "—".to_string() },
                Style::default().fg(Color::White),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "No module selected",
                Style::default().fg(Color::DarkGray),
            )));
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
