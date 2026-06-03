use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{EngineeringData, JournalData};
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
            let slot = format!("{:<28}", m.slot.clone());
            let item = format!("{:<48}", m.item.clone());
            let power = if m.power > 0.0 {
                format!("{:>6.2} MW", m.power)
            } else {
                format!("{:>9}", "—")
            };
            let cond = match m.health {
                Some(h) => format!("{:>5.1}%", h * 100.0),
                None    => "     —".to_string(),
            };
            let grade = match &m.engineering {
                Some(e) => format!(" G{}", e.level),
                None    => "   ".to_string(),
            };
            if selected {
                lines.push(Line::from(Span::styled(
                    format!(" {slot}  {item}  {power}  {cond}{grade}"),
                    Style::default().fg(Color::Black).bg(crate::theme::accent()).add_modifier(Modifier::BOLD),
                )));
            } else {
                let cond_color = condition_color(m.health);
                let grade_color = Color::Rgb(180, 100, 255);
                lines.push(Line::from(vec![
                    Span::styled(format!(" {slot}  {item}  {power}  "), Style::default().fg(Color::White)),
                    Span::styled(cond, Style::default().fg(cond_color)),
                    Span::styled(grade, Style::default().fg(grade_color)),
                ]));
            }
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
                    .border_style(Style::default().fg(crate::theme::accent())),
            ),
            area,
        );
    }

    fn render_detail(&self, frame: &mut Frame, area: Rect, journal: &JournalData) {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let p = &journal.pilot;
        if !p.ship_type.is_empty() {
            let orange = Style::default().fg(crate::theme::accent()).add_modifier(Modifier::BOLD);
            let cyan   = Style::default().fg(Color::Cyan);
            let white  = Style::default().fg(Color::White);
            let dim    = Style::default().fg(Color::DarkGray);
            let green  = Style::default().fg(Color::Green);
            let red    = Style::default().fg(Color::Red);

            lines.push(Line::from(Span::styled("Ship", orange)));
            lines.push(Line::from(vec![
                Span::styled("Type        ", cyan),
                Span::styled(p.ship_type.clone(), white),
            ]));
            if !p.ship_name.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Name        ", cyan),
                    Span::styled(p.ship_name.clone(), white.add_modifier(Modifier::BOLD)),
                ]));
            }
            if !p.ship_ident.is_empty() {
                lines.push(Line::from(vec![
                    Span::styled("Ident       ", cyan),
                    Span::styled(p.ship_ident.clone(), white),
                ]));
            }

            if p.hull_health > 0.0 {
                let pct = (p.hull_health * 100.0) as u8;
                let color = if pct > 75 { Color::Green }
                            else if pct > 50 { Color::Yellow }
                            else if pct > 25 { crate::theme::accent() }
                            else { Color::Red };
                lines.push(Line::from(vec![
                    Span::styled("Hull        ", cyan),
                    Span::styled(format!("{pct}%"), Style::default().fg(color)),
                ]));
            }

            if p.max_jump_range > 0.0 {
                lines.push(Line::from(vec![
                    Span::styled("Max Jump    ", cyan),
                    Span::styled(format!("{:.2} ly", p.max_jump_range), green),
                ]));
            }

            if p.unladen_mass > 0.0 {
                lines.push(Line::from(vec![
                    Span::styled("Unladen     ", cyan),
                    Span::styled(format!("{:.1} T", p.unladen_mass), white),
                ]));
            }

            if p.cargo_capacity > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Cargo       ", cyan),
                    Span::styled(format!("{} T", p.cargo_capacity), white),
                ]));
            }

            if p.fuel_capacity > 0.0 {
                let pct = if p.fuel_capacity > 0.0 {
                    (p.fuel_level / p.fuel_capacity * 100.0) as u8
                } else { 0 };
                let fuel_str = if p.reserve_fuel_capacity > 0.0 {
                    format!("{:.1}/{:.1}T + {:.2}T res  {}%",
                        p.fuel_level, p.fuel_capacity, p.reserve_fuel_capacity, pct)
                } else {
                    format!("{:.1}/{:.1}T  {}%", p.fuel_level, p.fuel_capacity, pct)
                };
                lines.push(Line::from(vec![
                    Span::styled("Fuel        ", cyan),
                    Span::styled(fuel_str, Style::default().fg(if pct > 25 { Color::Green } else { Color::Red })),
                ]));
            }

            if p.modules_value > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Modules     ", cyan),
                    Span::styled(format!("{} Cr", fmt_credits(p.modules_value)), white),
                ]));
            }

            if p.rebuy > 0 {
                lines.push(Line::from(vec![
                    Span::styled("Rebuy       ", cyan),
                    Span::styled(format!("{} Cr", fmt_credits(p.rebuy)), red),
                ]));
            }

            lines.push(Line::from(Span::styled("─".repeat(30), dim)));
            lines.push(Line::from(""));
        }

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
                Style::default().fg(crate::theme::accent()),
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
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Condition",
                Style::default().fg(Color::Cyan),
            )));
            let (cond_str, cond_color) = match m.health {
                Some(h) => (format!("{:.1}%", h * 100.0), condition_color(m.health)),
                None    => ("—".to_string(), Color::DarkGray),
            };
            lines.push(Line::from(Span::styled(cond_str, Style::default().fg(cond_color))));
            if let Some(v) = m.value {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    "Value",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(Span::styled(
                    format!("{} Cr", fmt_credits(v)),
                    Style::default().fg(Color::White),
                )));
            }
            if let Some(ref eng) = m.engineering {
                for l in engineering_lines(eng) {
                    lines.push(l);
                }
            }
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

fn engineering_lines(eng: &EngineeringData) -> Vec<Line<'static>> {
    let purple = Style::default().fg(Color::Rgb(180, 100, 255));
    let cyan   = Style::default().fg(Color::Cyan);
    let dim    = Style::default().fg(Color::DarkGray);

    let grade_dots = "●".repeat(eng.level as usize) + &"○".repeat(5usize.saturating_sub(eng.level as usize));
    let header = if eng.experimental.is_empty() {
        format!("{} G{}  {}", eng.blueprint, eng.level, grade_dots)
    } else {
        format!("{} G{}  {}", eng.blueprint, eng.level, grade_dots)
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(Span::styled("─".repeat(30), dim)),
        Line::from(Span::styled(header, purple.add_modifier(Modifier::BOLD))),
        Line::from(vec![
            Span::styled("By  ", cyan),
            Span::styled(eng.engineer.clone(), Style::default().fg(Color::White)),
        ]),
    ];

    if !eng.experimental.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Exp ", cyan),
            Span::styled(eng.experimental.clone(), Style::default().fg(Color::Rgb(255, 200, 80))),
        ]));
    }

    if !eng.modifiers.is_empty() {
        lines.push(Line::from(""));
        for m in &eng.modifiers {
            let improved = if m.less_is_good { m.value < m.original_value } else { m.value > m.original_value };
            let color = if improved { Color::Green } else { Color::Red };
            let arrow = if m.value > m.original_value { "▲" } else { "▼" };
            let pct = if m.original_value != 0.0 {
                ((m.value - m.original_value) / m.original_value.abs() * 100.0).round() as i32
            } else { 0 };
            lines.push(Line::from(vec![
                Span::styled(format!("{:<18}", m.label), dim),
                Span::styled(
                    format!("{:>8.2} → {:>8.2}  {}{:+}%", m.original_value, m.value, arrow, pct),
                    Style::default().fg(color),
                ),
            ]));
        }
    }

    lines
}

fn fmt_credits(v: i64) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 { out.push(','); }
        out.push(c);
    }
    out.chars().rev().collect()
}

fn condition_color(health: Option<f32>) -> Color {
    match health {
        Some(h) if h > 0.75 => Color::Green,
        Some(h) if h > 0.50 => Color::Yellow,
        Some(h) if h > 0.25 => crate::theme::accent(),
        Some(_) => Color::Red,
        None => Color::DarkGray,
    }
}
