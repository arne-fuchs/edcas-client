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
pub struct BodyInfo {
    pub name: String,
    pub body_id: i32,
    pub planet_class: String,
    pub landable: bool,
    pub terraform_state: String,
    pub volcanism: String,
    pub atmosphere: String,
    pub atmosphere_type: String,
    pub distance_from_arrival_ls: f32,
    pub radius: f32,
    pub mass_em: f32,
    pub surface_temperature: f32,
    pub surface_pressure: f32,
    pub surface_gravity: f32,
    pub tidal_lock: bool,
    pub rings: Vec<RingInfo>,
    pub materials: Vec<MaterialInfo>,
    pub signals: Vec<SignalInfo>,
    pub value: Option<i64>,
    pub estimated_value: i64,
}

#[derive(Clone)]
pub struct RingInfo {
    pub name: String,
    pub ring_class: String,
    pub mass_min: f32,
    pub mass_max: f32,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub has_hotspot: bool,
}

#[derive(Clone)]
pub struct MaterialInfo {
    pub name: String,
    pub percent: f32,
}

#[derive(Clone)]
pub struct SignalInfo {
    pub signal_type: String,
    pub count: i32,
}

pub struct MiningView {
    bodies: Vec<BodyInfo>,
    selected_body_idx: usize,
    scroll_offset: usize,
    show_ring_details: bool,
    show_materials: bool,
    show_signals: bool,
}

impl MiningView {
    pub fn new() -> Self {
        let mut view = Self {
            bodies: Vec::new(),
            selected_body_idx: 0,
            scroll_offset: 0,
            show_ring_details: true,
            show_materials: true,
            show_signals: true,
        };
        view.load_sample_data();
        view
    }

    fn load_sample_data(&mut self) {
        self.bodies.push(BodyInfo {
            name: "Shinrarta Dezhra 7".to_string(),
            body_id: 7,
            planet_class: "Metal Rich body".to_string(),
            landable: true,
            terraform_state: "Not terraformable".to_string(),
            volcanism: "Major Rocky Magma volcanism".to_string(),
            atmosphere: "No atmosphere".to_string(),
            atmosphere_type: "No atmosphere".to_string(),
            distance_from_arrival_ls: 45.23,
            radius: 4956000.0,
            mass_em: 0.5738,
            surface_temperature: 557.0,
            surface_pressure: 0.0,
            surface_gravity: 1.58,
            tidal_lock: false,
            rings: vec![
                RingInfo {
                    name: "Shinrarta Dezhra 7 A Ring".to_string(),
                    ring_class: "Metallic".to_string(),
                    mass_min: 590850000000.0,
                    mass_max: 23554000000000.0,
                    inner_radius: 826620000.0,
                    outer_radius: 1056830000.0,
                    has_hotspot: true,
                },
                RingInfo {
                    name: "Shinrarta Dezhra 7 B Ring".to_string(),
                    ring_class: "Metallic".to_string(),
                    mass_min: 148810000000000.0,
                    mass_max: 1106100000000000.0,
                    inner_radius: 1400470000.0,
                    outer_radius: 2424190000.0,
                    has_hotspot: true,
                },
                RingInfo {
                    name: "Shinrarta Dezhra 7 C Ring".to_string(),
                    ring_class: "Icy".to_string(),
                    mass_min: 43220000000000.0,
                    mass_max: 2208900000000000.0,
                    inner_radius: 3696100000.0,
                    outer_radius: 6752870000.0,
                    has_hotspot: false,
                },
            ],
            materials: vec![
                MaterialInfo { name: "Iron".to_string(), percent: 22.3 },
                MaterialInfo { name: "Nickel".to_string(), percent: 16.9 },
                MaterialInfo { name: "Sulphur".to_string(), percent: 16.3 },
                MaterialInfo { name: "Carbon".to_string(), percent: 14.2 },
                MaterialInfo { name: "Phosphorus".to_string(), percent: 9.3 },
                MaterialInfo { name: "Manganese".to_string(), percent: 7.4 },
                MaterialInfo { name: "Germanium".to_string(), percent: 5.8 },
                MaterialInfo { name: "Vanadium".to_string(), percent: 3.9 },
                MaterialInfo { name: "Zinc".to_string(), percent: 2.3 },
                MaterialInfo { name: "Arsenic".to_string(), percent: 1.5 },
            ],
            signals: vec![
                SignalInfo { signal_type: "Low Temperature Diamonds".to_string(), count: 3 },
                SignalInfo { signal_type: "Painite".to_string(), count: 5 },
                SignalInfo { signal_type: "Void Opal".to_string(), count: 2 },
                SignalInfo { signal_type: "Alexandrite".to_string(), count: 1 },
                SignalInfo { signal_type: "Rhodplumsite".to_string(), count: 4 },
            ],
            value: Some(12000),
            estimated_value: 324587,
        });

        self.bodies.push(BodyInfo {
            name: "Shinrarta Dezhra 4".to_string(),
            body_id: 4,
            planet_class: "High metal content body".to_string(),
            landable: true,
            terraform_state: "Candidate for terraforming".to_string(),
            volcanism: "Minor Silicate Vapour Geysers".to_string(),
            atmosphere: "Thin Argon-rich atmosphere".to_string(),
            atmosphere_type: "Argon-rich".to_string(),
            distance_from_arrival_ls: 12.87,
            radius: 3825000.0,
            mass_em: 0.1247,
            surface_temperature: 233.0,
            surface_pressure: 0.12,
            surface_gravity: 0.35,
            tidal_lock: false,
            rings: vec![],
            materials: vec![
                MaterialInfo { name: "Iron".to_string(), percent: 18.7 },
                MaterialInfo { name: "Nickel".to_string(), percent: 14.2 },
                MaterialInfo { name: "Sulphur".to_string(), percent: 13.8 },
                MaterialInfo { name: "Carbon".to_string(), percent: 12.0 },
                MaterialInfo { name: "Phosphorus".to_string(), percent: 7.9 },
            ],
            signals: vec![],
            value: Some(12000),
            estimated_value: 45892,
        });
    }

    fn build_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if self.bodies.is_empty() {
            lines.push(Line::from("No body scan data available. Scan planets and rings to view mining data."));
            return lines;
        }

        for (idx, body) in self.bodies.iter().enumerate() {
            let selected_marker = if idx == self.selected_body_idx { ">> " } else { "   " };

            lines.push(Line::from(Span::styled(
                format!("{}[Body {}] {}", selected_marker, body.body_id, body.name),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )));

            lines.push(Line::from(format!("  Class: {}", body.planet_class)));
            lines.push(Line::from(format!("  Distance: {:.2} Ls", body.distance_from_arrival_ls)));
            lines.push(Line::from(format!("  Radius: {:.0} m, Mass: {:.4} EM", body.radius, body.mass_em)));
            lines.push(Line::from(format!("  Gravity: {:.2}g, Temp: {:.0}K, Pressure: {:.2} atm",
                body.surface_gravity, body.surface_temperature, body.surface_pressure)));
            lines.push(Line::from(format!("  Landable: {}, Tidal Lock: {}",
                bool_icon(body.landable), bool_icon(body.tidal_lock))));
            lines.push(Line::from(format!("  Terraform State: {}", body.terraform_state)));
            lines.push(Line::from(format!("  Volcanism: {}", body.volcanism)));
            lines.push(Line::from(format!("  Atmosphere: {}", body.atmosphere)));
            lines.push(Line::from(format!("  Estimated Value: {} Cr", format_thousands(body.estimated_value))));
            lines.push(Line::from(""));

            if self.show_signals && !body.signals.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Surface Signals (Hotspots)",
                    Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                )));
                for signal in &body.signals {
                    let hotspot_style = if signal.count > 3 {
                        Style::default().fg(Color::Red)
                    } else if signal.count > 1 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("    {}: {} signal(s)", signal.signal_type, signal.count),
                        hotspot_style,
                    )));
                }
                lines.push(Line::from(""));
            }

            if self.show_materials && !body.materials.is_empty() {
                lines.push(Line::from(Span::styled(
                    "  Materials",
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
                    "  Rings",
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )));
                for ring in &body.rings {
                    let hotspot_indicator = if ring.has_hotspot { " [HOTSPOT]" } else { "" };
                    lines.push(Line::from(Span::styled(
                        format!("    {} ({}){}", ring.name, ring.ring_class, hotspot_indicator),
                        if ring.has_hotspot {
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
                if self.selected_body_idx > 0 {
                    self.selected_body_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                let max_idx = self.bodies.len().saturating_sub(1);
                if self.selected_body_idx < max_idx {
                    self.selected_body_idx += 1;
                }
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

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let lines = self.build_lines();

        let content_height = lines.len();
        let visible_height = area.height.saturating_sub(2) as usize;

        let mut paragraph = Paragraph::new(lines)
            .block(
                Block::default()
                    .title(" Mining (r:Rings, m:Materials, h:Hotspots) ")
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

fn material_bar(percent: f32) -> String {
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
