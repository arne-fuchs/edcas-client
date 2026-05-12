use crate::event_shim::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::journal_reader::JournalData;
use crate::views::ViewEvent;

#[derive(Clone, PartialEq)]
pub enum MaterialTab {
    Raw,
    Manufactured,
    Encoded,
    All,
}

impl MaterialTab {
    fn labels() -> Vec<&'static str> {
        vec!["Raw", "Manufactured", "Encoded", "All"]
    }

    fn next(&self) -> Self {
        match self {
            MaterialTab::Raw => MaterialTab::Manufactured,
            MaterialTab::Manufactured => MaterialTab::Encoded,
            MaterialTab::Encoded => MaterialTab::All,
            MaterialTab::All => MaterialTab::Raw,
        }
    }

    fn prev(&self) -> Self {
        match self {
            MaterialTab::Raw => MaterialTab::All,
            MaterialTab::Manufactured => MaterialTab::Raw,
            MaterialTab::Encoded => MaterialTab::Manufactured,
            MaterialTab::All => MaterialTab::Encoded,
        }
    }
}

pub struct MaterialsView {
    active_tab: MaterialTab,
    selected_idx: usize,
    scroll_offset: usize,
}

const RAW_MATERIALS: &[&str] = &[
    "iron", "nickel", "carbon", "phosphorus", "sulphur", "arsenic", "chromium",
    "germanium", "zinc", "zirconium", "vanadium", "manganese", "niobium", "tin",
    "tungsten", "antimony", "polonium", "ruthenium", "tellurium", "selenium",
    "yttrium", "cadmium", "mercury", "molybdenum", "technetium",
];

impl MaterialsView {
    pub fn new() -> Self {
        Self {
            active_tab: MaterialTab::All,
            selected_idx: 0,
            scroll_offset: 0,
        }
    }

    fn get_materials(journal: &JournalData) -> Vec<MaterialAggregate> {
        let mut material_map: std::collections::HashMap<String, MaterialAggregate> = std::collections::HashMap::new();

        for body in &journal.bodies {
            for mat in &body.materials {
                let entry = material_map.entry(mat.name.clone()).or_insert_with(|| MaterialAggregate {
                    name: mat.name.clone(),
                    total_percent: 0.0,
                    body_count: 0,
                    max_percent: 0.0,
                    max_body: String::new(),
                    category: categorize_material(&mat.name),
                });
                entry.total_percent += mat.percent;
                entry.body_count += 1;
                if mat.percent > entry.max_percent {
                    entry.max_percent = mat.percent;
                    entry.max_body = body.body_name.clone();
                }
            }
        }

        let mut materials: Vec<MaterialAggregate> = material_map.into_values().collect();
        materials.sort_by(|a, b| b.max_percent.partial_cmp(&a.max_percent).unwrap_or(std::cmp::Ordering::Equal));
        materials
    }

    fn build_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let all_materials = Self::get_materials(journal);

        let materials: Vec<&MaterialAggregate> = all_materials.iter()
            .filter(|m| {
                match self.active_tab {
                    MaterialTab::All => true,
                    MaterialTab::Raw => m.category == "Raw",
                    MaterialTab::Manufactured => m.category == "Manufactured",
                    MaterialTab::Encoded => m.category == "Encoded",
                }
            })
            .collect();

        if materials.is_empty() {
            lines.push(Line::from("No materials found in scanned bodies."));
            lines.push(Line::from(""));
            lines.push(Line::from("Detailed scans will reveal surface materials."));
            return lines;
        }

        lines.push(Line::from(Span::styled(
            format!("Materials in System ({})", materials.len()),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        let mut current_category: Option<String> = None;
        let mut local_idx = 0;

        for mat in &materials {
            if Some(&mat.category) != current_category.as_ref() {
                current_category = Some(mat.category.clone());
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }
                let category_color = match mat.category.as_str() {
                    "Raw" => Color::Green,
                    "Manufactured" => Color::Yellow,
                    "Encoded" => Color::Cyan,
                    _ => Color::White,
                };
                lines.push(Line::from(Span::styled(
                    format!("{} Materials", mat.category),
                    Style::default().fg(category_color).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  {:<18} {:>5} {:>15}  {}", "Material", "% Max", "Found On", "Distribution"),
                    Style::default().fg(Color::DarkGray),
                )));
                local_idx = 0;
            }

            let is_selected = local_idx == self.selected_idx;
            let marker = if is_selected { ">> " } else { "   " };

            let bar = material_bar(mat.max_percent);
            let bar_color = if mat.max_percent > 20.0 {
                Color::Green
            } else if mat.max_percent > 10.0 {
                Color::Yellow
            } else {
                Color::White
            };

            let style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(bar_color)
            };

            lines.push(Line::from(Span::styled(
                format!("  {}{:<18} {:>5.1}% {:>15}  {}", marker, capitalize(&mat.name), mat.max_percent, mat.max_body, bar),
                style,
            )));

            local_idx += 1;
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Total unique materials: {}", materials.len()),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));

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
            KeyCode::Char('a') | KeyCode::Left => {
                self.active_tab = self.active_tab.prev();
                self.selected_idx = 0;
            }
            KeyCode::Char('d') | KeyCode::Right => {
                self.active_tab = self.active_tab.next();
                self.selected_idx = 0;
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
                    .title(" Materials (a/d: tab) ")
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

#[derive(Clone)]
struct MaterialAggregate {
    name: String,
    total_percent: f64,
    body_count: usize,
    max_percent: f64,
    max_body: String,
    category: String,
}

fn categorize_material(name: &str) -> String {
    let lower = name.to_lowercase();
    if RAW_MATERIALS.iter().any(|r| lower.contains(r)) {
        "Raw".to_string()
    } else if lower.contains("encoded") || lower.contains("data") {
        "Encoded".to_string()
    } else if lower.contains("component") || lower.contains("plate") || lower.contains("assembly")
        || lower.contains("conductor") || lower.contains("coil") || lower.contains("bus")
        || lower.contains("refined") || lower.contains("configured") {
        "Manufactured".to_string()
    } else {
        "Raw".to_string()
    }
}

fn material_bar(percent: f64) -> String {
    let filled = (percent / 5.0).round() as usize;
    let empty = 20 - filled.min(20);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
    }
}
