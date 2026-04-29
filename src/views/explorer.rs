use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::views::ViewEvent;

#[derive(Clone)]
pub struct SystemInfo {
    pub name: String,
    pub coords: (f32, f32, f32),
    pub system_address: i64,
    pub economy: String,
    pub second_economy: String,
    pub government: String,
    pub allegiance: String,
    pub security: String,
    pub population: i64,
    pub controlling_power: Option<String>,
    pub powers: Vec<String>,
    pub factions: Vec<FactionInfo>,
}

#[derive(Clone)]
pub struct FactionInfo {
    pub name: String,
    pub government: String,
    pub influence: f32,
    pub happiness: String,
    pub active_states: Vec<String>,
    pub recovering_states: Vec<String>,
    pub pending_states: Vec<String>,
}

struct GridRow {
    cells: Vec<CellType>,
}

enum CellType {
    Label(String),
    Value(String),
    Header(String),
    Section(String),
}

pub struct ExplorerView {
    current_system: Option<SystemInfo>,
    visited_systems: Vec<SystemInfo>,
    selected_system_idx: usize,
    scroll_offset: usize,
}

impl ExplorerView {
    pub fn new() -> Self {
        let mut view = Self {
            current_system: None,
            visited_systems: Vec::new(),
            selected_system_idx: 0,
            scroll_offset: 0,
        };
        view.load_sample_data();
        view
    }

    fn load_sample_data(&mut self) {
        self.current_system = Some(SystemInfo {
            name: "Shinrarta Dezhra".to_string(),
            coords: (49.81, -14.06, 8.81),
            system_address: 2715745104002,
            economy: "High Tech".to_string(),
            second_economy: "Refinery".to_string(),
            government: "Democracy".to_string(),
            allegiance: "Independent".to_string(),
            security: "High".to_string(),
            population: 17_605_427_128,
            controlling_power: Some("Zachary Hudson".to_string()),
            powers: vec!["Zachary Hudson".to_string(), "Pranav Antal".to_string()],
            factions: vec![
                FactionInfo {
                    name: "Pilots Federation Local Branch".to_string(),
                    government: "Democracy".to_string(),
                    influence: 0.475,
                    happiness: "Happy".to_string(),
                    active_states: vec!["None".to_string()],
                    recovering_states: vec![],
                    pending_states: vec![],
                },
                FactionInfo {
                    name: "Shinrarta Industry Inc".to_string(),
                    government: "Corporate".to_string(),
                    influence: 0.203,
                    happiness: "Happy".to_string(),
                    active_states: vec!["Boom".to_string()],
                    recovering_states: vec![],
                    pending_states: vec!["Expansion".to_string()],
                },
                FactionInfo {
                    name: "Shinrarta's Pioneers".to_string(),
                    government: "Patronage".to_string(),
                    influence: 0.148,
                    happiness: "Happy".to_string(),
                    active_states: vec!["None".to_string()],
                    recovering_states: vec![],
                    pending_states: vec![],
                },
            ],
        });
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if let Some(system) = &self.current_system {
            lines.push(Line::from(Span::styled(
                format!("Current System: {}", system.name),
                Style::default()
                    .fg(Color::Yellow)
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
            lines.push(Line::from(format!("  Economy: {}", system.economy)));
            lines.push(Line::from(format!("  Secondary Economy: {}", system.second_economy)));
            lines.push(Line::from(format!("  Government: {}", system.government)));
            lines.push(Line::from(format!("  Allegiance: {}", system.allegiance)));
            lines.push(Line::from(format!("  Security: {}", system.security)));
            lines.push(Line::from(format!("  Population: {}", format_population(system.population))));
            lines.push(Line::from(""));

            if let Some(ref power) = system.controlling_power {
                lines.push(Line::from(Span::styled(
                    "Powerplay",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::UNDERLINED),
                )));
                lines.push(Line::from(format!("  Controlling Power: {}", power)));
                lines.push(Line::from(format!("  Powers Present: {}", system.powers.join(", "))));
                lines.push(Line::from(""));
            }

            lines.push(Line::from(Span::styled(
                "Factions",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::UNDERLINED),
            )));
            for (i, faction) in system.factions.iter().enumerate() {
                let active = if i == 0 { " (Controlling)" } else { "" };
                lines.push(Line::from(Span::styled(
                    format!("  {}{}", faction.name, active),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(format!("    Government: {}, Influence: {:.1}%, Happiness: {}",
                    faction.government, faction.influence * 100.0, faction.happiness)));
                if !faction.active_states.is_empty() {
                    lines.push(Line::from(format!("    Active States: {}", faction.active_states.join(", "))));
                }
                if !faction.pending_states.is_empty() {
                    lines.push(Line::from(format!("    Pending States: {}", faction.pending_states.join(", "))));
                }
                lines.push(Line::from(""));
            }
        } else {
            lines.push(Line::from("No system data available."));
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_system_idx > 0 {
                    self.selected_system_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                let max_idx = self.visited_systems.len().saturating_sub(1);
                if self.selected_system_idx < max_idx {
                    self.selected_system_idx += 1;
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self.build_lines();

        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;

        let mut paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Explorer ")
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
