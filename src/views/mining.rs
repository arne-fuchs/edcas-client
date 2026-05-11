use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::journal_reader::{JournalData, BodyScan};
use crate::views::ViewEvent;

pub struct MiningView {
    selected_idx: usize,
    scroll_offset: usize,
    show_ring_details: bool,
    show_materials: bool,
    show_signals: bool,
}

impl MiningView {
    pub fn new() -> Self {
        Self {
            selected_idx: 0,
            scroll_offset: 0,
            show_ring_details: true,
            show_materials: true,
            show_signals: true,
        }
    }

    fn get_bodies(journal: &JournalData) -> Vec<&BodyScan> {
        journal.bodies.iter()
            .filter(|b| b.landable || !b.rings.is_empty() || !b.materials.is_empty())
            .collect()
    }

    fn build_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let bodies = Self::get_bodies(journal);

        if bodies.is_empty() {
            lines.push(Line::from("No mineable bodies found in current system."));
            lines.push(Line::from(""));
            lines.push(Line::from("Scan planets with surface materials, rings, or hotspots."));
            return lines;
        }

        for (idx, body) in bodies.iter().enumerate() {
            let selected_marker = if idx == self.selected_idx { ">> " } else { "   " };

            lines.push(Line::from(Span::styled(
                format!("{}[Body {}] {}", selected_marker, body.body_id, body.body_name),
                Style::default()
                    .fg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )));

            lines.push(Line::from(format!("  Class: {}", body.planet_class)));
            lines.push(Line::from(format!("  Distance: {:.2} Ls", body.distance_from_arrival_ls)));
            lines.push(Line::from(format!("  Radius: {:.0} m, Mass: {:.4} EM", body.radius, body.mass_em)));
            lines.push(Line::from(format!("  Gravity: {:.2}g, Temp: {:.0}K",
                body.surface_gravity, body.surface_temperature)));
            lines.push(Line::from(format!("  Landable: {}, Tidal Lock: {}",
                bool_icon(body.landable), bool_icon(body.tidal_lock))));
            if !body.terraform_state.is_empty() {
                lines.push(Line::from(format!("  Terraform: {}", body.terraform_state)));
            }
            if !body.volcanism.is_empty() {
                lines.push(Line::from(format!("  Volcanism: {}", body.volcanism)));
            }
            if !body.atmosphere.is_empty() {
                lines.push(Line::from(format!("  Atmosphere: {}", body.atmosphere)));
            }
            if body.estimated_value > 0 {
                lines.push(Line::from(format!("  Estimated Value: {} Cr", format_thousands(body.estimated_value))));
            }
            lines.push(Line::from(""));

            if self.show_materials && !body.materials.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Surface Materials",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )));
                for mat in &body.materials {
                    let bar = material_bar(mat.percent);
                    lines.push(Line::from(format!("    {:<15} {:>5.1}% {}", mat.name, mat.percent, bar)));
                }
                lines.push(Line::from(""));
            }

            if self.show_ring_details && !body.rings.is_empty() {
                lines.push(Line::from(Span::styled(
                    format!("  Rings ({})", body.rings.len()),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                for ring in &body.rings {
                    let class_label = clean_ring_class(&ring.ring_class);
                    let hotspot_hint = if is_valuable_ring_class(&ring.ring_class) { " [VALUABLE]" } else { "" };
                    lines.push(Line::from(Span::styled(
                        format!("    {} ({}){}", ring.name, class_label, hotspot_hint),
                        if is_valuable_ring_class(&ring.ring_class) {
                            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::White)
                        },
                    )));
                }
                lines.push(Line::from(""));
            }
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                self.selected_idx += 1;
            }
            KeyCode::Char('r') => {
                self.show_ring_details = !self.show_ring_details;
            }
            KeyCode::Char('m') => {
                self.show_materials = !self.show_materials;
            }
            KeyCode::Char('h') => {
                self.show_signals = !self.show_signals;
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
                    .title(" Mining (r:Rings, m:Materials) ")
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

fn bool_icon(val: bool) -> &'static str {
    if val { "Yes" } else { "No" }
}

fn material_bar(percent: f64) -> String {
    let filled = (percent / 10.0).round() as usize;
    let empty = 10 - filled.min(10);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn format_thousands(n: i64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let len = s.len();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

fn clean_ring_class(class: &str) -> &str {
    class.strip_prefix("eRingClass_").unwrap_or(class)
}

fn is_valuable_ring_class(class: &str) -> bool {
    let lower = class.to_lowercase();
    lower.contains("metal") || lower.contains("rocky")
}
