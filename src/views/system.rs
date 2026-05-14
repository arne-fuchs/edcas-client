use std::cmp::Ordering;

use crate::event_shim::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::journal_reader::{JournalData, ConflictData};
use crate::views::ViewEvent;

pub struct SystemView {
    selected_faction: usize,
}

impl SystemView {
    pub fn new() -> Self {
        Self { selected_faction: 0 }
    }

    /// Returns rendered lines plus the starting line index of each faction
    /// (in influence-descending order), so render can auto-scroll to the selection.
    fn build_lines(&self, journal: &JournalData) -> (Vec<Line<'static>>, Vec<usize>) {
        let mut lines = Vec::new();
        let mut faction_starts: Vec<usize> = Vec::new();

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
                Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
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

            let has_power = system.controlling_power.as_ref().map(|p| !p.is_empty()).unwrap_or(false)
                || !system.powers.is_empty();
            if has_power {
                lines.push(Line::from(Span::styled(
                    "Powerplay",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                )));
                if let Some(ref power) = system.controlling_power {
                    if !power.is_empty() {
                        lines.push(Line::from(format!("  Controlling Power: {}", power)));
                    }
                }
                if !system.powers.is_empty() {
                    lines.push(Line::from(format!("  Powers Present: {}", system.powers.join(", "))));
                }
                lines.push(Line::from(""));
            }

            if !system.factions.is_empty() {
                let mut sorted = system.factions.clone();
                sorted.sort_by(|a, b| b.influence.partial_cmp(&a.influence).unwrap_or(Ordering::Equal));

                lines.push(Line::from(Span::styled(
                    format!("Factions ({}) — w/s: select  space: open in Factions tab", sorted.len()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                )));

                let selected = self.selected_faction.min(sorted.len().saturating_sub(1));

                for (i, faction) in sorted.iter().enumerate() {
                    faction_starts.push(lines.len());

                    let is_selected = i == selected;
                    let is_controlling = faction.name == system.system_faction;

                    let name_style = if is_selected {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Rgb(255, 140, 0))
                            .add_modifier(Modifier::BOLD)
                    } else if is_controlling {
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    };

                    let tag = if is_controlling { " ★" } else { "" };
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {:<36}{}", faction.name, tag), name_style),
                    ]));

                    let pct = faction.influence * 100.0;
                    let filled = (faction.influence * 20.0).round() as usize;
                    let bar_color = if pct < 15.0 {
                        Color::Red
                    } else if pct < 40.0 {
                        Color::Yellow
                    } else {
                        Color::Green
                    };
                    lines.push(Line::from(vec![
                        Span::raw(format!("    Influence: {:>5.1}%  [", pct)),
                        Span::styled("█".repeat(filled), Style::default().fg(bar_color)),
                        Span::styled("░".repeat(20 - filled), Style::default().fg(Color::DarkGray)),
                        Span::raw("]"),
                    ]));

                    let mut state_parts: Vec<String> = Vec::new();
                    if !faction.active_states.is_empty() {
                        state_parts.push(format!("Active: {}", faction.active_states.join(", ")));
                    }
                    if !faction.pending_states.is_empty() {
                        state_parts.push(format!("Pending: {}", faction.pending_states.join(", ")));
                    }
                    if !faction.recovering_states.is_empty() {
                        state_parts.push(format!("Recovering: {}", faction.recovering_states.join(", ")));
                    }
                    if !state_parts.is_empty() {
                        lines.push(Line::from(Span::styled(
                            format!("    {}", state_parts.join("  |  ")),
                            Style::default().fg(Color::DarkGray),
                        )));
                    }

                    if let Some(ref c) = faction.conflict {
                        for l in conflict_lines(c) {
                            lines.push(l);
                        }
                    }

                    lines.push(Line::from(""));
                }
            }
        } else {
            lines.push(Line::from("No system data available."));
            lines.push(Line::from(""));
            lines.push(Line::from("Jump to a system to see information here."));
        }

        (lines, faction_starts)
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        let faction_count = journal
            .current_system
            .as_ref()
            .map(|s| s.factions.len())
            .unwrap_or(0);

        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if faction_count > 0 {
                    self.selected_faction = self.selected_faction.saturating_sub(1);
                    return ViewEvent::Consumed;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if faction_count > 0 {
                    self.selected_faction = (self.selected_faction + 1).min(faction_count - 1);
                    return ViewEvent::Consumed;
                }
            }
            KeyCode::Char(' ') => {
                if let Some(system) = &journal.current_system {
                    if !system.factions.is_empty() {
                        let mut sorted = system.factions.clone();
                        sorted.sort_by(|a, b| {
                            b.influence.partial_cmp(&a.influence).unwrap_or(Ordering::Equal)
                        });
                        let idx = self.selected_faction.min(sorted.len() - 1);
                        return ViewEvent::OpenFactions(sorted[idx].name.clone());
                    }
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let (lines, faction_starts) = self.build_lines(journal);

        let visible_height = area.height.saturating_sub(2) as usize;
        let content_height = lines.len();

        // auto-scroll to keep the selected faction in view
        let scroll = if !faction_starts.is_empty() {
            let selected = self.selected_faction.min(faction_starts.len() - 1);
            faction_starts[selected].saturating_sub(1)
        } else {
            0
        };
        let max_scroll = content_height.saturating_sub(visible_height);

        frame.render_widget(
            Paragraph::new(lines)
                .block(
                    Block::default()
                        .title(" System ")
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White)),
                )
                .scroll((scroll.min(max_scroll) as u16, 0)),
            area,
        );
    }
}

fn conflict_lines(c: &ConflictData) -> Vec<Line<'static>> {
    let type_label = match c.war_type.to_lowercase().as_str() {
        "war" => "War",
        "civilwar" => "Civil War",
        "election" => "Election",
        _ => "Conflict",
    };
    let status = if c.status.is_empty() { "active".to_string() } else { c.status.clone() };

    let score_color = Color::Rgb(255, 140, 0);
    let dim = Color::DarkGray;

    let mut lines = vec![
        Line::from(vec![
            Span::styled("    ".to_string(), Style::default()),
            Span::styled(format!("{} ({})", type_label, status), Style::default().fg(score_color)),
            Span::styled("  vs  ".to_string(), Style::default().fg(dim)),
            Span::styled(c.opponent.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled(
                format!("    Score: {} – {}", c.our_won_days, c.opponent_won_days),
                Style::default().fg(score_color),
            ),
        ]),
    ];

    if !c.our_stake.is_empty() || !c.opponent_stake.is_empty() {
        let stake_str = match (c.our_stake.is_empty(), c.opponent_stake.is_empty()) {
            (false, false) => format!("    Stakes: {} / {}", c.our_stake, c.opponent_stake),
            (false, true)  => format!("    Stake: {}", c.our_stake),
            (true, false)  => format!("    Opponent stake: {}", c.opponent_stake),
            (true, true)   => unreachable!(),
        };
        lines.push(Line::from(Span::styled(stake_str, Style::default().fg(dim))));
    }

    lines
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
