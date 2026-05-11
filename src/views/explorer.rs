use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::debug;
use crate::journal_reader::{JournalData, BodyScan, TreeNode, build_body_tree, BodyComposition as JournalBodyComposition};
use crate::views::ViewEvent;

pub struct ExplorerView {
    tree: Vec<TreeNode>,
    selected_idx: usize,
    flat_nodes: Vec<FlatNode>,
    scroll_offset: usize,
    system_name: String,
}

struct FlatNode {
    /// ASCII tree drawing prefix, e.g. "│  ├─ "
    tree_prefix: String,
    /// Body name with system-name prefix stripped
    short_name: String,
    body_id: i32,
    distance_ls: f32,
    has_rings: bool,
    landable: bool,
    planet_class: String,
    star_type: String,
    is_barycentre: bool,
    composition: Option<JournalBodyComposition>,
}

impl ExplorerView {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            selected_idx: 0,
            flat_nodes: Vec::new(),
            scroll_offset: 0,
            system_name: String::new(),
        }
    }

    pub fn update(&mut self, journal: &JournalData) {
        let tree = build_body_tree(&journal.bodies);
        debug!("Explorer updated with {} bodies, {} tree nodes", journal.bodies.len(), tree.len());
        self.system_name = journal.current_system
            .as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_default();
        self.tree = tree;
        self.rebuild_flat_nodes();
        if self.selected_idx >= self.flat_nodes.len() {
            self.selected_idx = self.flat_nodes.len().saturating_sub(1);
        }
    }

    fn rebuild_flat_nodes(&mut self) {
        self.flat_nodes.clear();
        let n = self.tree.len();
        for (i, node) in self.tree.iter().enumerate() {
            flatten_node(node, "", i == n - 1, &mut self.flat_nodes, &self.system_name);
        }
    }

    fn get_selected_body(&self) -> Option<&BodyScan> {
        let node_id = self.flat_nodes.get(self.selected_idx)?.body_id;
        find_body_in_tree(&self.tree, node_id)
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        let prev = self.selected_idx;
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_idx + 1 < self.flat_nodes.len() {
                    self.selected_idx += 1;
                }
            }
            KeyCode::PageUp => {
                self.selected_idx = self.selected_idx.saturating_sub(10);
            }
            KeyCode::PageDown => {
                self.selected_idx =
                    (self.selected_idx + 10).min(self.flat_nodes.len().saturating_sub(1));
            }
            KeyCode::Home => {
                self.selected_idx = 0;
            }
            KeyCode::End => {
                self.selected_idx = self.flat_nodes.len().saturating_sub(1);
            }
            _ => {}
        }
        let _ = prev;
        ViewEvent::None
    }

    /// Keep the selected tree row within the visible window.
    /// Line 0 is the system header; flat_nodes[i] is at line i+1.
    fn auto_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        let selected_line = self.selected_idx + 1;
        if selected_line < self.scroll_offset {
            self.scroll_offset = selected_line;
        } else if selected_line >= self.scroll_offset + visible_height {
            self.scroll_offset = selected_line + 1 - visible_height;
        }
    }

    fn build_tree_lines(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // System header
        let header_text = if self.system_name.is_empty() {
            "Unknown System".to_string()
        } else {
            self.system_name.clone()
        };
        lines.push(Line::from(Span::styled(
            header_text,
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .add_modifier(Modifier::BOLD),
        )));

        if self.flat_nodes.is_empty() {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "No bodies scanned yet.",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "Use the FSS scanner to map bodies.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        for (idx, node) in self.flat_nodes.iter().enumerate() {
            let is_selected = idx == self.selected_idx;

            let (icon, icon_style) = node_icon(node);
            let icon_style = if is_selected {
                icon_style.add_modifier(Modifier::BOLD)
            } else {
                icon_style
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                icon_style
            };

            let prefix_style = if is_selected {
                Style::default().fg(Color::Rgb(255, 140, 0))
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let dist_str = if node.distance_ls > 0.0 {
                format!("  {:>8.1} Ls", node.distance_ls)
            } else {
                String::new()
            };

            let mut hints = String::new();
            if node.has_rings {
                hints.push_str("  ⌀");
            }
            if node.landable {
                hints.push_str("  L");
            }

            // Show star type in parentheses if available
            let display_name = if !node.star_type.is_empty() {
                format!("{} ({})", node.short_name, node.star_type)
            } else {
                node.short_name.clone()
            };

            let mut spans: Vec<Span<'static>> = vec![
                Span::styled(node.tree_prefix.clone(), prefix_style),
                Span::styled(format!("{} ", icon), icon_style),
                Span::styled(display_name, name_style),
                Span::styled(dist_str, Style::default().fg(Color::DarkGray)),
            ];

            if !hints.is_empty() {
                spans.push(Span::styled(hints, Style::default().fg(Color::Cyan)));
            }

            lines.push(Line::from(spans));
        }

        lines
    }

    fn build_detail_lines(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let Some(body) = self.get_selected_body() else {
            lines.push(Line::from(Span::styled(
                "Select a body to see details.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        };

        // Name
        lines.push(Line::from(Span::styled(
            body.body_name.clone(),
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .add_modifier(Modifier::BOLD),
        )));

        // Type
        let type_label = if !body.planet_class.is_empty() {
            body.planet_class.clone()
        } else if !body.star_type.is_empty() {
            format!("Star ({})", body.star_type)
        } else {
            "Star / Barycentre".to_string()
        };
        lines.push(Line::from(Span::styled(
            type_label,
            Style::default().fg(Color::White),
        )));
        lines.push(Line::from(""));

        // Physical
        lines.push(section_header("Physical"));
        detail_row(&mut lines, "Distance", format!("{:.2} Ls", body.distance_from_arrival_ls));
        if body.radius > 0.0 {
            detail_row(&mut lines, "Radius", format!("{:.0} km", body.radius / 1000.0));
        }
        if body.mass_em > 0.0 {
            detail_row(&mut lines, "Mass", format!("{:.4} EM", body.mass_em));
        }
        if body.surface_gravity > 0.0 {
            detail_row(&mut lines, "Gravity", format!("{:.2} g", body.surface_gravity));
        }
        if body.surface_temperature > 0.0 {
            detail_row(&mut lines, "Temperature", format!("{:.0} K", body.surface_temperature));
        }
        lines.push(Line::from(""));

        // Status
        lines.push(section_header("Status"));
        detail_row(&mut lines, "Landable", if body.landable { "Yes".into() } else { "No".into() });
        if body.tidal_lock {
            detail_row(&mut lines, "Tidal Lock", "Yes".into());
        }
        if !body.terraform_state.is_empty() {
            detail_row(&mut lines, "Terraform", body.terraform_state.clone());
        }
        if !body.atmosphere.is_empty() {
            detail_row(&mut lines, "Atmosphere", body.atmosphere.clone());
        }
        if !body.volcanism.is_empty() {
            detail_row(&mut lines, "Volcanism", body.volcanism.clone());
        }
        if !body.scan_type.is_empty() {
            detail_row(&mut lines, "Scan", body.scan_type.clone());
        }
        if body.estimated_value > 0 {
            detail_row(
                &mut lines,
                "Value",
                format!("{} Cr", format_thousands(body.estimated_value)),
            );
        }

        // Rings
        if !body.rings.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header(&format!("Rings ({})", body.rings.len())));
            for ring in &body.rings {
                let short = ring
                    .name
                    .strip_prefix(&body.body_name)
                    .unwrap_or(&ring.name)
                    .trim()
                    .to_string();
                let class = ring
                    .ring_class
                    .strip_prefix("eRingClass_")
                    .unwrap_or(&ring.ring_class)
                    .replace('_', " ");
                lines.push(Line::from(vec![
                    Span::styled("  ⌀ ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:<14}", short), Style::default().fg(Color::White)),
                    Span::styled(class, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        // Composition
        if let Some(comp) = &body.composition {
            lines.push(Line::from(""));
            lines.push(section_header("Composition"));
            detail_row(&mut lines, "Ice", format!("{:.1}%", comp.ice * 100.0));
            detail_row(&mut lines, "Rock", format!("{:.1}%", comp.rock * 100.0));
            detail_row(&mut lines, "Metal", format!("{:.1}%", comp.metal * 100.0));
        }


        // Materials
        if !body.materials.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header(&format!("Materials ({})", body.materials.len())));
            let mut mats = body.materials.clone();
            mats.sort_by(|a, b| {
                b.percent
                    .partial_cmp(&a.percent)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            for mat in &mats {
                let name = capitalise(&mat.name);
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<18}", name), Style::default().fg(Color::White)),
                    Span::styled(
                        format!("{:>5.1}%", mat.percent),
                        Style::default().fg(Color::Rgb(255, 140, 0)),
                    ),
                ]));
            }
        }

        lines
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let tree_area = chunks[0];
        let detail_area = chunks[1];

        // Tree panel
        let visible_height = tree_area.height.saturating_sub(2) as usize;
        self.auto_scroll(visible_height);
        let tree_lines = self.build_tree_lines();

        let tree_paragraph = Paragraph::new(tree_lines)
            .block(
                Block::default()
                    .title(" Navigation (w/s ↑↓  PgUp/PgDn) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            )
            .scroll((self.scroll_offset as u16, 0));
        frame.render_widget(tree_paragraph, tree_area);

        // Detail panel
        let detail_lines = self.build_detail_lines();
        let detail_paragraph = Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .title(" Body Details ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White)),
            );
        frame.render_widget(detail_paragraph, detail_area);
    }
}

// ── Tree flattening ──────────────────────────────────────────────────────────

fn flatten_node(
    node: &TreeNode,
    parent_prefix: &str,
    is_last: bool,
    result: &mut Vec<FlatNode>,
    system_name: &str,
) {
    let connector = if is_last { "└─ " } else { "├─ " };
    let continuation = if is_last { "   " } else { "│  " };
    let tree_prefix = format!("{}{}", parent_prefix, connector);
    let child_prefix = format!("{}{}", parent_prefix, continuation);

    let body = node.data.as_ref();
    let is_barycentre = node.name.to_ascii_lowercase().contains("barycentre");
    let short_name = strip_system_prefix(&node.name, system_name);

    result.push(FlatNode {
        tree_prefix,
        short_name,
        body_id: node.body_id,
        distance_ls: body.map(|b| b.distance_from_arrival_ls).unwrap_or(0.0),
        has_rings: body.map(|b| !b.rings.is_empty()).unwrap_or(false),
        landable: body.map(|b| b.landable).unwrap_or(false),
        planet_class: body.map(|b| b.planet_class.clone()).unwrap_or_default(),
        star_type: body.map(|b| b.star_type.clone()).unwrap_or_default(),
        is_barycentre,
        composition: body.and_then(|b| b.composition.as_ref().map(|c| JournalBodyComposition {
            ice: c.ice,
            rock: c.rock,
            metal: c.metal,
        })),
    });

    let n = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        flatten_node(child, &child_prefix, i == n - 1, result, system_name);
    }
}

fn find_body_in_tree<'a>(nodes: &'a [TreeNode], body_id: i32) -> Option<&'a BodyScan> {
    for node in nodes {
        if node.body_id == body_id {
            return node.data.as_ref();
        }
        if let Some(found) = find_body_in_tree(&node.children, body_id) {
            return Some(found);
        }
    }
    None
}

// ── Icons & styling ──────────────────────────────────────────────────────────

fn node_icon(node: &FlatNode) -> (&'static str, Style) {
    if node.planet_class.is_empty() {
        if node.is_barycentre {
            ("⊕", Style::default().fg(Color::DarkGray))
        } else {
            ("★", Style::default().fg(Color::Rgb(255, 140, 0)))
        }
    } else {
        body_class_icon(&node.planet_class)
    }
}

fn body_class_icon(planet_class: &str) -> (&'static str, Style) {
    let lower = planet_class.to_lowercase();
    if lower.contains("earthlike") {
        ("●", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
    } else if lower.contains("water world") || lower.contains("water giant") {
        ("●", Style::default().fg(Color::Blue))
    } else if lower.contains("ammonia") {
        ("●", Style::default().fg(Color::LightMagenta))
    } else if lower.contains("metal rich") {
        ("●", Style::default().fg(Color::Red))
    } else if lower.contains("high metal") {
        ("●", Style::default().fg(Color::LightRed))
    } else if lower.contains("icy") {
        ("●", Style::default().fg(Color::Cyan))
    } else if lower.contains("rocky") {
        ("●", Style::default().fg(Color::Gray))
    } else if lower.contains("gas") || lower.contains("sudarsky") || lower.contains("giant") {
        ("○", Style::default().fg(Color::Rgb(255, 165, 0)))
    } else {
        ("●", Style::default().fg(Color::White))
    }
}

// ── Detail panel helpers ─────────────────────────────────────────────────────

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("─ {} ", title),
        Style::default()
            .fg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD),
    ))
}

fn detail_row(lines: &mut Vec<Line<'static>>, label: &str, value: String) {
    lines.push(Line::from(vec![
        Span::styled(
            format!("  {:<14}", label),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled(value, Style::default().fg(Color::White)),
    ]));
}

// ── Utilities ────────────────────────────────────────────────────────────────

fn strip_system_prefix(name: &str, system_name: &str) -> String {
    if system_name.is_empty() || name == system_name {
        return name.to_string();
    }
    if let Some(rest) = name.strip_prefix(system_name) {
        let trimmed = rest.trim_start();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    name.to_string()
}

fn capitalise(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
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
