use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::journal_reader::JournalData;
use crate::views::ViewEvent;

pub struct SystemView {
    scroll_offset: usize,
}

impl SystemView {
    pub fn new() -> Self {
        Self {
            scroll_offset: 0,
        }
    }

    fn build_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if let Some(system) = &journal.current_system {
            lines.push(Line::from(Span::styled(
                format!("System: {}", system.name),
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )));
            lines.push(Line::from(""));

            lines.push(Line::from(Span::styled(
                "System Information",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            lines.push(Line::from(format!("  Coordinates: ({:.2}, {:.2}, {:.2})",
                system.coords.0, system.coords.1, system.coords.2)));
            lines.push(Line::from(format!("  System Address: {}", system.system_address)));
            lines.push(Line::from(format!("  Current Body: {} ({})", system.body, system.body_type)));
            lines.push(Line::from(format!("  Economy: {}", system.economy)));
            if !system.second_economy.is_empty() {
                lines.push(Line::from(format!("  Secondary Economy: {}", system.second_economy)));
            }
            lines.push(Line::from(format!("  Government: {}", system.government)));
            lines.push(Line::from(format!("  Allegiance: {}", system.allegiance)));
            lines.push(Line::from(format!("  Security: {}", system.security)));
            lines.push(Line::from(format!("  Population: {}", format_population(system.population))));
            lines.push(Line::from(""));

            if !system.controlling_power.as_ref().unwrap_or(&String::new()).is_empty() || !system.powers.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Powerplay",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                )));
                if let Some(ref power) = system.controlling_power {
                    lines.push(Line::from(format!("  Controlling Power: {}", power)));
                }
                if !system.powers.is_empty() {
                    lines.push(Line::from(format!("  Powers Present: {}", system.powers.join(", "))));
                }
                lines.push(Line::from(""));
            }

            if !system.factions.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("Factions ({})", system.factions.len()),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                )));
                for faction in system.factions.iter() {
                    let controlling = if faction == &system.system_faction { " [Controlling]" } else { "" };
                    lines.push(Line::from(Span::styled(
                        format!("  {}{}", faction, controlling),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )));
                }
                lines.push(Line::from(""));
            }
        } else {
            lines.push(Line::from("No system data available."));
            lines.push(Line::from(""));
            lines.push(Line::from("Jump to a system to see information here."));
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent, _journal: &JournalData) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.scroll_offset > 0 {
                    self.scroll_offset -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.scroll_offset += 1;
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let lines = self.build_lines(journal);

        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;

        let mut paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" System ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );

        if content_height > visible_height {
            let max_scroll = content_height.saturating_sub(visible_height);
            paragraph = paragraph.scroll((self.scroll_offset.min(max_scroll) as u16, 0));
        }

        frame.render_widget(paragraph, area);
    }
}

fn format_population(pop: i64) -> String {
    if pop >= 1_000_000_000 {
        format!("{:.2}B", pop as f64 / 1_000_000_000.0)
    } else if pop >= 1_000_000 {
        format!("{:.2}M", pop as f64 / 1_000_000.0)
    } else if pop >= 1_000 {
        format!("{:.2}K", pop as f64 / 1_000.0)
    } else {
        pop.to_string()
    }
}
