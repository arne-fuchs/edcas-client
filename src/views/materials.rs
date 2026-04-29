use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use crate::views::ViewEvent;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub category: MaterialCategory,
    pub count: i32,
    pub limit: i32,
    pub grade: Option<i32>,
}

#[derive(Clone, PartialEq)]
pub enum MaterialCategory {
    Raw,
    Manufactured,
    Encoded,
    Human,
    Guardian,
}

impl MaterialCategory {
    fn as_str(&self) -> &'static str {
        match self {
            MaterialCategory::Raw => "Raw",
            MaterialCategory::Manufactured => "Manufactured",
            MaterialCategory::Encoded => "Encoded",
            MaterialCategory::Human => "Human Tech",
            MaterialCategory::Guardian => "Guardian Tech",
        }
    }

    fn color(&self) -> Color {
        match self {
            MaterialCategory::Raw => Color::Green,
            MaterialCategory::Manufactured => Color::Yellow,
            MaterialCategory::Encoded => Color::Cyan,
            MaterialCategory::Human => Color::LightRed,
            MaterialCategory::Guardian => Color::LightMagenta,
        }
    }
}

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

    fn matches(&self, category: &MaterialCategory) -> bool {
        match (self, category) {
            (MaterialTab::Raw, MaterialCategory::Raw) => true,
            (MaterialTab::Manufactured, MaterialCategory::Manufactured) => true,
            (MaterialTab::Encoded, MaterialCategory::Encoded) => true,
            (MaterialTab::All, _) => true,
            _ => false,
        }
    }
}

pub struct MaterialsView {
    materials: Vec<Material>,
    selected_idx: usize,
    active_tab: MaterialTab,
    scroll_offset: usize,
}

impl MaterialsView {
    pub fn new() -> Self {
        let mut view = Self {
            materials: Vec::new(),
            selected_idx: 0,
            active_tab: MaterialTab::Raw,
            scroll_offset: 0,
        };
        view.load_sample_data();
        view
    }

    fn load_sample_data(&mut self) {
        self.materials = vec![
            // Raw Materials
            Material { name: "Iron".to_string(), category: MaterialCategory::Raw, count: 156, limit: 300, grade: None },
            Material { name: "Nickel".to_string(), category: MaterialCategory::Raw, count: 142, limit: 300, grade: None },
            Material { name: "Carbon".to_string(), category: MaterialCategory::Raw, count: 89, limit: 300, grade: None },
            Material { name: "Phosphorus".to_string(), category: MaterialCategory::Raw, count: 67, limit: 300, grade: None },
            Material { name: "Sulphur".to_string(), category: MaterialCategory::Raw, count: 124, limit: 300, grade: None },
            Material { name: "Arsenic".to_string(), category: MaterialCategory::Raw, count: 45, limit: 300, grade: None },
            Material { name: "Chromium".to_string(), category: MaterialCategory::Raw, count: 78, limit: 300, grade: None },
            Material { name: "Germanium".to_string(), category: MaterialCategory::Raw, count: 56, limit: 300, grade: None },
            Material { name: "Zinc".to_string(), category: MaterialCategory::Raw, count: 92, limit: 300, grade: None },
            Material { name: "Zirconium".to_string(), category: MaterialCategory::Raw, count: 34, limit: 300, grade: None },
            Material { name: "Vanadium".to_string(), category: MaterialCategory::Raw, count: 67, limit: 300, grade: None },
            Material { name: "Manganese".to_string(), category: MaterialCategory::Raw, count: 112, limit: 300, grade: None },
            Material { name: "Niobium".to_string(), category: MaterialCategory::Raw, count: 28, limit: 300, grade: None },
            Material { name: "Tin".to_string(), category: MaterialCategory::Raw, count: 45, limit: 300, grade: None },
            Material { name: "Tungsten".to_string(), category: MaterialCategory::Raw, count: 23, limit: 300, grade: None },
            Material { name: "Antimony".to_string(), category: MaterialCategory::Raw, count: 18, limit: 300, grade: None },
            Material { name: "Polonium".to_string(), category: MaterialCategory::Raw, count: 8, limit: 300, grade: None },
            Material { name: "Ruthenium".to_string(), category: MaterialCategory::Raw, count: 15, limit: 300, grade: None },
            Material { name: "Tellurium".to_string(), category: MaterialCategory::Raw, count: 12, limit: 300, grade: None },
            Material { name: "Selenium".to_string(), category: MaterialCategory::Raw, count: 34, limit: 300, grade: None },
            Material { name: "Yttrium".to_string(), category: MaterialCategory::Raw, count: 6, limit: 300, grade: None },
            Material { name: "Cadmium".to_string(), category: MaterialCategory::Raw, count: 9, limit: 300, grade: None },
            Material { name: "Mercury".to_string(), category: MaterialCategory::Raw, count: 4, limit: 300, grade: None },
            // Manufactured Materials
            Material { name: "Refined Focus Crystals".to_string(), category: MaterialCategory::Manufactured, count: 24, limit: 300, grade: Some(5) },
            Material { name: "Configurable Components".to_string(), category: MaterialCategory::Manufactured, count: 45, limit: 300, grade: Some(4) },
            Material { name: "Mechanical Components".to_string(), category: MaterialCategory::Manufactured, count: 67, limit: 300, grade: Some(4) },
            Material { name: "Heat Dispersion Plate".to_string(), category: MaterialCategory::Manufactured, count: 18, limit: 300, grade: Some(5) },
            Material { name: "Heat Vanes".to_string(), category: MaterialCategory::Manufactured, count: 34, limit: 300, grade: Some(4) },
            Material { name: "Hydrogen Fuel".to_string(), category: MaterialCategory::Manufactured, count: 89, limit: 300, grade: Some(3) },
            Material { name: "Chemical Manipulators".to_string(), category: MaterialCategory::Manufactured, count: 23, limit: 300, grade: Some(4) },
            Material { name: "Electrode Arrays".to_string(), category: MaterialCategory::Manufactured, count: 56, limit: 300, grade: Some(3) },
            Material { name: "Propulsion Elements".to_string(), category: MaterialCategory::Manufactured, count: 45, limit: 300, grade: Some(3) },
            Material { name: "Worn Shield Emitters".to_string(), category: MaterialCategory::Manufactured, count: 78, limit: 300, grade: Some(2) },
            Material { name: "Damaged Escape Pod".to_string(), category: MaterialCategory::Manufactured, count: 12, limit: 300, grade: Some(5) },
            Material { name: "Galactic Travel Equipment".to_string(), category: MaterialCategory::Manufactured, count: 34, limit: 300, grade: Some(3) },
            // Encoded Materials
            Material { name: "Encoded Emissions Data".to_string(), category: MaterialCategory::Encoded, count: 23, limit: 300, grade: None },
            Material { name: "Divergent Scan Data".to_string(), category: MaterialCategory::Encoded, count: 15, limit: 300, grade: None },
            Material { name: "Abnormal Scan Data".to_string(), category: MaterialCategory::Encoded, count: 8, limit: 300, grade: None },
            Material { name: "Classified Scan Data".to_string(), category: MaterialCategory::Encoded, count: 34, limit: 300, grade: None },
            Material { name: "Anomalous Bulk Scan Data".to_string(), category: MaterialCategory::Encoded, count: 12, limit: 300, grade: None },
            Material { name: "Atypical Disrupted Wake Echoes".to_string(), category: MaterialCategory::Encoded, count: 5, limit: 300, grade: None },
        ];
    }

    fn filtered_materials(&self) -> Vec<&Material> {
        self.materials.iter()
            .filter(|m| self.active_tab.matches(&m.category))
            .collect()
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let filtered = self.filtered_materials();

        if filtered.is_empty() {
            lines.push(Line::from("No materials in this category."));
            return lines;
        }

        let mut current_category: Option<MaterialCategory> = None;
        let mut local_idx = 0;

        for mat in &filtered {
            if Some(mat.category.clone()) != current_category {
                current_category = Some(mat.category.clone());
                if !lines.is_empty() {
                    lines.push(Line::from(""));
                }
                lines.push(Line::from(Span::styled(
                    format!("{} Materials", mat.category.as_str()),
                    Style::default()
                        .fg(mat.category.color())
                        .add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  {:<30} {:>6} / {:<6}  {}", "Name", "Count", "Limit", "Capacity"),
                    Style::default().fg(Color::DarkGray),
                )));
                local_idx = 0;
            }

            let is_selected = local_idx == self.selected_idx;
            let marker = if is_selected { ">> " } else { "   " };

            let ratio = mat.count as f32 / mat.limit as f32;
            let bar = material_bar(ratio);
            let _bar_color = if ratio > 0.8 { Color::Red } else if ratio > 0.5 { Color::Yellow } else { Color::Green };

            let grade_str = if let Some(grade) = mat.grade {
                format!("(G{})", grade)
            } else {
                String::new()
            };

            let style = if is_selected {
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(mat.category.color())
            };

            lines.push(Line::from(Span::styled(
                format!("  {}{:<30} {:>6} / {:<6}  {}", marker, format!("{} {}", mat.name, grade_str), mat.count, mat.limit, bar),
                style,
            )));

            local_idx += 1;
        }

        lines.push(Line::from(""));
        let total: i32 = self.materials.iter().map(|m| m.count).sum();
        let total_limit: i32 = self.materials.iter().map(|m| m.limit).sum();
        lines.push(Line::from(Span::styled(
            format!("Total: {} / {} materials", total, total_limit),
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
        )));

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        let filtered = self.filtered_materials();
        let max_idx = filtered.len().saturating_sub(1);

        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_idx < max_idx {
                    self.selected_idx += 1;
                }
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

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self.build_lines();

        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;

        let _tabs = Tabs::new(MaterialTab::labels())
            .select(match self.active_tab {
                MaterialTab::Raw => 0,
                MaterialTab::Manufactured => 1,
                MaterialTab::Encoded => 2,
                MaterialTab::All => 3,
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .divider("|");

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

fn material_bar(ratio: f32) -> String {
    let filled = (ratio * 10.0).round() as usize;
    let empty = 10 - filled.min(10);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}
