use std::cmp::Ordering;

use crate::event_shim::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::journal_reader::{JournalData, SystemData, ConflictData};
use crate::views::util::FocusArea;
use crate::views::ViewEvent;

pub struct SystemView {
    selected_system_idx: usize,
    selected_faction: usize,
    detail_scroll: usize,
    focus: FocusArea,
}

impl SystemView {
    pub fn new() -> Self {
        Self {
            selected_system_idx: 0,
            selected_faction: 0,
            detail_scroll: 0,
            focus: FocusArea::Detail,
        }
    }

    fn build_list_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if journal.visited_systems.is_empty() {
            lines.push(Line::from(Span::styled("No systems visited yet.", Style::default().fg(Color::DarkGray))));
            return lines;
        }
        for (i, sys) in journal.visited_systems.iter().enumerate() {
            let is_current = i == 0;
            let is_selected = i == self.selected_system_idx;
            let style = if is_selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_current { "\u{2299} " } else { "  " };
            lines.push(Line::from(Span::styled(format!("{}{}", prefix, sys.name), style)));
        }
        lines
    }

    fn build_detail_lines(&self, system: &SystemData, journal: &JournalData) -> (Vec<Line<'static>>, Vec<usize>) {
        let mut lines = Vec::new();
        let mut faction_starts: Vec<usize> = Vec::new();

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

        // Colonisation section — only for the currently occupied system (index 0)
        if self.selected_system_idx == 0 {
            let depots_in_system: Vec<_> = journal.construction_depots.values()
                .filter(|d| d.system_name == system.name)
                .collect();
            let is_architect = journal.claimed_systems.contains_key(&system.system_address);

            if is_architect || !depots_in_system.is_empty() {
                lines.push(Line::from(Span::styled(
                    "Colonisation",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
                )));
                if is_architect {
                    let cmdr = if journal.pilot.name.is_empty() {
                        "You".to_string()
                    } else {
                        format!("CMDR {}", journal.pilot.name)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("  System Architect: {}", cmdr),
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                    )));
                }
                for depot in &depots_in_system {
                    let pct = depot.submission.progress * 100.0;
                    let filled = (depot.submission.progress * 20.0).round() as usize;
                    let bar_color = if pct >= 100.0 {
                        Color::Green
                    } else if pct > 50.0 {
                        Color::Yellow
                    } else {
                        Color::Cyan
                    };
                    let remaining = depot.submission.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .count();
                    lines.push(Line::from(vec![
                        Span::styled(
                            format!("  \u{2605} {}", depot.submission.station_name),
                            Style::default().fg(Color::White),
                        ),
                    ]));
                    lines.push(Line::from(vec![
                        Span::raw(format!("    Progress: {:>5.1}%  [", pct)),
                        Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
                        Span::styled("\u{2591}".repeat(20 - filled), Style::default().fg(Color::DarkGray)),
                        Span::raw(format!("]  {} commodities needed", remaining)),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }

        if !system.factions.is_empty() {
            let mut sorted = system.factions.clone();
            sorted.sort_by(|a, b| b.influence.partial_cmp(&a.influence).unwrap_or(Ordering::Equal));

            lines.push(Line::from(Span::styled(
                format!("Factions ({}) \u{2014} w/s: select  space: open in Factions tab", sorted.len()),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
            )));

            let selected = self.selected_faction.min(sorted.len().saturating_sub(1));

            for (i, faction) in sorted.iter().enumerate() {
                faction_starts.push(lines.len());

                let is_selected = self.focus == FocusArea::Detail && i == selected;
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

                let tag = if is_controlling { " \u{2605}" } else { "" };
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
                    Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
                    Span::styled("\u{2591}".repeat(20 - filled), Style::default().fg(Color::DarkGray)),
                    Span::raw("]"),
                ]));

                let mut state_parts: Vec<String> = Vec::new();
                if !faction.active_states.is_empty() {
                    let annotated: Vec<String> = faction.active_states.iter()
                        .map(|s| super::annotate_faction_state(s, false))
                        .collect();
                    state_parts.push(format!("Active: {}", annotated.join(", ")));
                }
                if !faction.pending_states.is_empty() {
                    let annotated: Vec<String> = faction.pending_states.iter()
                        .map(|s| super::annotate_faction_state(s, true))
                        .collect();
                    state_parts.push(format!("Pending: {}", annotated.join(", ")));
                }
                if !faction.recovering_states.is_empty() {
                    let annotated: Vec<String> = faction.recovering_states.iter()
                        .map(|s| super::annotate_faction_state(s, false))
                        .collect();
                    state_parts.push(format!("Recovering: {}", annotated.join(", ")));
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

        (lines, faction_starts)
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        let system_count = journal.visited_systems.len();

        let faction_count = journal.visited_systems
            .get(self.selected_system_idx)
            .map(|s| s.factions.len())
            .unwrap_or(0);

        match self.focus {
            FocusArea::List => {
                match key.code {
                    KeyCode::Char('w') | KeyCode::Up => {
                        if system_count > 0 && self.selected_system_idx > 0 {
                            self.selected_system_idx -= 1;
                            self.selected_faction = 0;
                            self.detail_scroll = 0;
                        }
                        return ViewEvent::Consumed;
                    }
                    KeyCode::Char('s') | KeyCode::Down => {
                        if self.selected_system_idx + 1 < system_count {
                            self.selected_system_idx += 1;
                            self.selected_faction = 0;
                            self.detail_scroll = 0;
                        }
                        return ViewEvent::Consumed;
                    }
                    KeyCode::PageUp => {
                        self.selected_system_idx = self.selected_system_idx.saturating_sub(10);
                        self.selected_faction = 0;
                        self.detail_scroll = 0;
                        return ViewEvent::Consumed;
                    }
                    KeyCode::PageDown => {
                        self.selected_system_idx = (self.selected_system_idx + 10)
                            .min(system_count.saturating_sub(1));
                        self.selected_faction = 0;
                        self.detail_scroll = 0;
                        return ViewEvent::Consumed;
                    }
                    KeyCode::Tab | KeyCode::Char('d') | KeyCode::Right | KeyCode::Enter => {
                        if system_count > 0 {
                            self.focus = FocusArea::Detail;
                        }
                        return ViewEvent::Consumed;
                    }
                    _ => {}
                }
            }
            FocusArea::Detail => {
                match key.code {
                    KeyCode::Char('w') | KeyCode::Up => {
                        if faction_count > 0 {
                            self.selected_faction = self.selected_faction.saturating_sub(1);
                        } else {
                            self.detail_scroll = self.detail_scroll.saturating_sub(1);
                        }
                        return ViewEvent::Consumed;
                    }
                    KeyCode::Char('s') | KeyCode::Down => {
                        if faction_count > 0 {
                            self.selected_faction = (self.selected_faction + 1).min(faction_count.saturating_sub(1));
                        } else {
                            self.detail_scroll += 1;
                        }
                        return ViewEvent::Consumed;
                    }
                    KeyCode::PageUp => {
                        self.detail_scroll = self.detail_scroll.saturating_sub(10);
                        return ViewEvent::Consumed;
                    }
                    KeyCode::PageDown => {
                        self.detail_scroll += 10;
                        return ViewEvent::Consumed;
                    }
                    KeyCode::Tab | KeyCode::Char('a') | KeyCode::Left | KeyCode::Esc => {
                        self.focus = FocusArea::List;
                        return ViewEvent::Consumed;
                    }
                    KeyCode::Char(' ') => {
                        if let Some(system) = journal.visited_systems.get(self.selected_system_idx) {
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
            }
        }

        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        // Left: system history list
        let list_lines = self.build_list_lines(journal);
        frame.render_widget(
            Paragraph::new(list_lines)
                .block(Block::default()
                    .title(" Systems ")
                    .borders(Borders::ALL)
                    .border_style(if self.focus == FocusArea::List { active_border } else { inactive_border })),
            chunks[0],
        );

        // Right: system detail
        let detail_block = Block::default()
            .title(" System \u{2014} a/tab: switch focus  w/s: factions  space: open faction ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail { active_border } else { inactive_border });

        if let Some(system) = journal.visited_systems.get(self.selected_system_idx) {
            let (lines, faction_starts) = self.build_detail_lines(system, journal);

            let visible_height = chunks[1].height.saturating_sub(2) as usize;
            let content_height = lines.len();

            let scroll = if self.focus == FocusArea::Detail && !faction_starts.is_empty() {
                let selected = self.selected_faction.min(faction_starts.len() - 1);
                faction_starts[selected].saturating_sub(1)
            } else {
                self.detail_scroll
            };
            let max_scroll = content_height.saturating_sub(visible_height);

            frame.render_widget(
                Paragraph::new(lines)
                    .block(detail_block)
                    .scroll((scroll.min(max_scroll) as u16, 0)),
                chunks[1],
            );
        } else {
            frame.render_widget(
                Paragraph::new(vec![
                    Line::from("No system data available."),
                    Line::from(""),
                    Line::from("Jump to a system to see information here."),
                ]).block(detail_block),
                chunks[1],
            );
        }
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
                format!("    Score: {} \u{2013} {}", c.our_won_days, c.opponent_won_days),
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
