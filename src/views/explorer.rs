use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::debug;

use crate::journal_reader::{
    BodyComposition as JournalBodyComposition, BodyScan, BodySignal, DiscoveredSignal, JournalData,
    SaaBodyData, StationData, TreeNode, build_body_tree,
};
use crate::views::ViewEvent;

pub struct ExplorerView {
    tree: Vec<TreeNode>,
    selected_idx: usize,
    flat_nodes: Vec<FlatNode>,
    scroll_offset: usize,
    system_name: String,
    fss_signals: HashMap<i32, Vec<BodySignal>>,
    saa_data: HashMap<i32, SaaBodyData>,
    stations: Vec<StationData>,
    discovered_signals: Vec<DiscoveredSignal>,
}

enum NodeType {
    Body,
    SectionHeader,
    Station(StationData),
    Signal(DiscoveredSignal),
}

struct FlatNode {
    tree_prefix: String,
    short_name: String,
    body_id: i32,
    distance_ls: f32,
    has_rings: bool,
    landable: bool,
    planet_class: String,
    star_type: String,
    is_barycentre: bool,
    composition: Option<JournalBodyComposition>,
    bio_signal_count: i32,
    geo_signal_count: i32,
    node_type: NodeType,
}

impl ExplorerView {
    pub fn new() -> Self {
        Self {
            tree: Vec::new(),
            selected_idx: 0,
            flat_nodes: Vec::new(),
            scroll_offset: 0,
            system_name: String::new(),
            fss_signals: HashMap::new(),
            saa_data: HashMap::new(),
            stations: Vec::new(),
            discovered_signals: Vec::new(),
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
        self.fss_signals = journal.fss_signals.clone();
        self.saa_data = journal.saa_data.clone();
        self.stations = journal.stations.clone();
        self.discovered_signals = journal.discovered_signals.clone();
        self.rebuild_flat_nodes();
        if self.selected_idx >= self.flat_nodes.len() {
            self.selected_idx = self.flat_nodes.len().saturating_sub(1);
        }
    }

    fn rebuild_flat_nodes(&mut self) {
        self.flat_nodes.clear();
        let n = self.tree.len();
        for (i, node) in self.tree.iter().enumerate() {
            flatten_node(
                node,
                "",
                i == n - 1,
                &mut self.flat_nodes,
                &self.system_name,
                &self.fss_signals,
                &self.saa_data,
            );
        }
        if !self.stations.is_empty() {
            self.flat_nodes.push(FlatNode {
                tree_prefix: String::new(),
                short_name: "Stations".into(),
                body_id: -1,
                distance_ls: 0.0,
                has_rings: false,
                landable: false,
                planet_class: String::new(),
                star_type: String::new(),
                is_barycentre: false,
                composition: None,
                bio_signal_count: 0,
                geo_signal_count: 0,
                node_type: NodeType::SectionHeader,
            });
            for station in &self.stations {
                self.flat_nodes.push(FlatNode {
                    tree_prefix: String::new(),
                    short_name: station.name.clone(),
                    body_id: -1,
                    distance_ls: station.dist_from_star_ls,
                    has_rings: false,
                    landable: false,
                    planet_class: String::new(),
                    star_type: String::new(),
                    is_barycentre: false,
                    composition: None,
                    bio_signal_count: 0,
                    geo_signal_count: 0,
                    node_type: NodeType::Station(station.clone()),
                });
            }
        }
        if !self.discovered_signals.is_empty() {
            self.flat_nodes.push(FlatNode {
                tree_prefix: String::new(),
                short_name: format!("Signals ({})", self.discovered_signals.len()),
                body_id: -1,
                distance_ls: 0.0,
                has_rings: false,
                landable: false,
                planet_class: String::new(),
                star_type: String::new(),
                is_barycentre: false,
                composition: None,
                bio_signal_count: 0,
                geo_signal_count: 0,
                node_type: NodeType::SectionHeader,
            });
            for sig in &self.discovered_signals {
                self.flat_nodes.push(FlatNode {
                    tree_prefix: String::new(),
                    short_name: sig.display_name.clone(),
                    body_id: -1,
                    distance_ls: 0.0,
                    has_rings: false,
                    landable: false,
                    planet_class: String::new(),
                    star_type: String::new(),
                    is_barycentre: false,
                    composition: None,
                    bio_signal_count: 0,
                    geo_signal_count: 0,
                    node_type: NodeType::Signal(sig.clone()),
                });
            }
        }
    }

    fn get_selected_body(&self) -> Option<&BodyScan> {
        let node = self.flat_nodes.get(self.selected_idx)?;
        if !matches!(node.node_type, NodeType::Body) {
            return None;
        }
        find_body_in_tree(&self.tree, node.body_id)
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
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
        ViewEvent::None
    }

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

            match &node.node_type {
                NodeType::SectionHeader => {
                    lines.push(Line::from(""));
                    let style = if is_selected {
                        Style::default()
                            .fg(Color::Rgb(255, 200, 100))
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                            .fg(Color::Rgb(255, 140, 0))
                            .add_modifier(Modifier::BOLD)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("─ {} ", node.short_name),
                        style,
                    )));
                }
                NodeType::Station(_) => {
                    let name_style = if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let icon_style = if is_selected {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::Cyan)
                    };
                    let dist_str = if node.distance_ls > 0.0 {
                        format!("  {:>8.1} Ls", node.distance_ls)
                    } else {
                        String::new()
                    };
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("◉ ", icon_style),
                        Span::styled(node.short_name.clone(), name_style),
                        Span::styled(dist_str, Style::default().fg(Color::DarkGray)),
                    ]));
                }
                NodeType::Signal(ref sig) => {
                    let (icon, base_color) = signal_icon(sig);
                    let icon_style = if is_selected {
                        Style::default().fg(base_color).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(base_color)
                    };
                    let name_style = if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    let threat_str = sig.threat_level
                        .filter(|&t| t > 0)
                        .map(|t| format!("  ⚠{}", t))
                        .unwrap_or_default();
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(format!("{} ", icon), icon_style),
                        Span::styled(node.short_name.clone(), name_style),
                        Span::styled(threat_str, Style::default().fg(Color::Red)),
                    ]));
                }
                NodeType::Body => {
                    let (icon, icon_style) = node_icon(node);
                    let icon_style = if is_selected {
                        icon_style.add_modifier(Modifier::BOLD)
                    } else {
                        icon_style
                    };
                    let name_style = if is_selected {
                        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
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
                    if node.bio_signal_count > 0 {
                        hints.push_str(&format!("  Bio:{}", node.bio_signal_count));
                    }
                    if node.geo_signal_count > 0 {
                        hints.push_str(&format!("  Geo:{}", node.geo_signal_count));
                    }
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
            }
        }

        lines
    }

    fn build_detail_lines(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if let Some(node) = self.flat_nodes.get(self.selected_idx) {
            if let NodeType::Station(ref station) = node.node_type {
                return build_station_detail(station);
            }
            if let NodeType::Signal(ref sig) = node.node_type {
                return build_signal_detail(sig);
            }
            if matches!(node.node_type, NodeType::SectionHeader) {
                return lines;
            }
        }

        let Some(body) = self.get_selected_body().cloned() else {
            lines.push(Line::from(Span::styled(
                "Select a body to see details.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        };

        lines.push(Line::from(Span::styled(
            body.body_name.clone(),
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .add_modifier(Modifier::BOLD),
        )));

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
            detail_row(&mut lines, "Value", format!("{} Cr", format_thousands(body.estimated_value)));
        }

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

        if let Some(comp) = &body.composition {
            lines.push(Line::from(""));
            lines.push(section_header("Composition"));
            detail_row(&mut lines, "Ice", format!("{:.1}%", comp.ice * 100.0));
            detail_row(&mut lines, "Rock", format!("{:.1}%", comp.rock * 100.0));
            detail_row(&mut lines, "Metal", format!("{:.1}%", comp.metal * 100.0));
        }

        if !body.materials.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header(&format!("Materials ({})", body.materials.len())));
            let mut mats = body.materials.clone();
            mats.sort_by(|a, b| b.percent.partial_cmp(&a.percent).unwrap_or(std::cmp::Ordering::Equal));
            for mat in &mats {
                let name = capitalise(&mat.name);
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<18}", name), Style::default().fg(Color::White)),
                    Span::styled(format!("{:>5.1}%", mat.percent), Style::default().fg(Color::Rgb(255, 140, 0))),
                ]));
            }
        }

        // Signals — prefer SAA (detailed) over FSS (counts only)
        let body_id = body.body_id;
        let is_nav_beacon = body.scan_type == "NavBeaconDetail";
        if let Some(saa) = self.saa_data.get(&body_id) {
            lines.push(Line::from(""));
            lines.push(section_header(&format!("Signals ({})", saa.signals.len())));
            for sig in &saa.signals {
                let type_name = sig.display_type().to_string();
                let sig_style = if sig.is_biological() {
                    Style::default().fg(Color::Green)
                } else if sig.is_geological() {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<18}", type_name), sig_style),
                    Span::styled(format!("{:>3}x", sig.count), Style::default().fg(Color::Rgb(255, 140, 0))),
                ]));
            }
            if !saa.genuses.is_empty() {
                lines.push(Line::from(""));
                lines.push(section_header("Biological Genera"));
                for genus in &saa.genuses {
                    lines.push(Line::from(vec![
                        Span::styled("  ", Style::default()),
                        Span::styled(genus.clone(), Style::default().fg(Color::Green)),
                    ]));
                }
            }
        } else if let Some(sigs) = self.fss_signals.get(&body_id) {
            if !sigs.is_empty() {
                lines.push(Line::from(""));
                lines.push(section_header(&format!("Signals ({})", sigs.len())));
                for sig in sigs {
                    let type_name = sig.display_type().to_string();
                    let sig_style = if sig.is_biological() {
                        Style::default().fg(Color::Green)
                    } else if sig.is_geological() {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {:<18}", type_name), sig_style),
                        Span::styled(format!("{:>3}x", sig.count), Style::default().fg(Color::Rgb(255, 140, 0))),
                    ]));
                }
            }
        } else if !body.planet_class.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header("Signals"));
            let hint = if is_nav_beacon {
                "NavBeacon doesn't include signal data."
            } else {
                "Use FSS scanner to detect signals."
            };
            lines.push(Line::from(Span::styled(
                format!("  {}", hint),
                Style::default().fg(Color::DarkGray),
            )));
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

        let detail_lines = self.build_detail_lines();
        let detail_paragraph = Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .title(" Details ")
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
    fss_signals: &HashMap<i32, Vec<BodySignal>>,
    saa_data: &HashMap<i32, SaaBodyData>,
) {
    let connector = if is_last { "└─ " } else { "├─ " };
    let continuation = if is_last { "   " } else { "│  " };
    let tree_prefix = format!("{}{}", parent_prefix, connector);
    let child_prefix = format!("{}{}", parent_prefix, continuation);

    let body = node.data.as_ref();
    let is_barycentre = node.name.to_ascii_lowercase().contains("barycentre");
    let short_name = strip_system_prefix(&node.name, system_name);

    let (bio_signal_count, geo_signal_count) = if let Some(saa) = saa_data.get(&node.body_id) {
        let bio = saa.signals.iter().filter(|s| s.is_biological()).map(|s| s.count).sum();
        let geo = saa.signals.iter().filter(|s| s.is_geological()).map(|s| s.count).sum();
        (bio, geo)
    } else if let Some(fss) = fss_signals.get(&node.body_id) {
        let bio = fss.iter().filter(|s| s.is_biological()).map(|s| s.count).sum();
        let geo = fss.iter().filter(|s| s.is_geological()).map(|s| s.count).sum();
        (bio, geo)
    } else {
        (0, 0)
    };

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
        bio_signal_count,
        geo_signal_count,
        node_type: NodeType::Body,
    });

    let n = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
        flatten_node(child, &child_prefix, i == n - 1, result, system_name, fss_signals, saa_data);
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

// ── Station detail ───────────────────────────────────────────────────────────

fn build_station_detail(station: &StationData) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(Span::styled(
        station.name.clone(),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        station.station_type.clone(),
        Style::default().fg(Color::White),
    )));
    lines.push(Line::from(""));

    lines.push(section_header("Location"));
    if station.dist_from_star_ls > 0.0 {
        detail_row(&mut lines, "Distance", format!("{:.2} Ls", station.dist_from_star_ls));
    }
    if !station.faction.is_empty() {
        detail_row(&mut lines, "Faction", station.faction.clone());
    }
    if !station.government.is_empty() {
        detail_row(&mut lines, "Government", station.government.clone());
    }
    if !station.economy.is_empty() {
        detail_row(&mut lines, "Economy", station.economy.clone());
    }

    if !station.services.is_empty() {
        lines.push(Line::from(""));
        lines.push(section_header(&format!("Services ({})", station.services.len())));
        for chunk in station.services.chunks(2) {
            lines.push(Line::from(Span::styled(
                format!("  {}", chunk.join("  •  ")),
                Style::default().fg(Color::White),
            )));
        }
    }

    lines
}

// ── Signal detail ────────────────────────────────────────────────────────────

fn build_signal_detail(sig: &DiscoveredSignal) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let (icon, color) = signal_icon(sig);
    lines.push(Line::from(vec![
        Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::styled(sig.display_name.clone(), Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    if let Some(ref uss_type) = sig.uss_type {
        lines.push(section_header("USS"));
        detail_row(&mut lines, "Type", uss_type.clone());
        if let Some(threat) = sig.threat_level {
            let threat_color = if threat >= 4 { Color::Red } else if threat >= 2 { Color::Rgb(255, 140, 0) } else { Color::Yellow };
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", "Threat"), Style::default().fg(Color::Cyan)),
                Span::styled(threat.to_string(), Style::default().fg(threat_color)),
            ]));
        }
        lines.push(Line::from(""));
    }

    if sig.spawning_faction.is_some() || sig.spawning_state.is_some() {
        lines.push(section_header("Origin"));
        if let Some(ref faction) = sig.spawning_faction {
            detail_row(&mut lines, "Faction", faction.clone());
        }
        if let Some(ref state) = sig.spawning_state {
            detail_row(&mut lines, "State", state.clone());
        }
        lines.push(Line::from(""));
    }

    if sig.is_station {
        detail_row(&mut lines, "Type", "Station / Installation".into());
    }

    if let Some(remaining) = sig.time_remaining {
        let mins = (remaining / 60.0) as u32;
        let secs = (remaining % 60.0) as u32;
        detail_row(&mut lines, "Expires in", format!("{}m {}s", mins, secs));
    }

    lines
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

fn signal_icon(sig: &DiscoveredSignal) -> (&'static str, Color) {
    if sig.uss_type.is_some() {
        let threat = sig.threat_level.unwrap_or(0);
        if threat >= 4 {
            ("⚠", Color::Red)
        } else if threat >= 2 {
            ("⚠", Color::Rgb(255, 140, 0))
        } else {
            ("⚠", Color::Yellow)
        }
    } else if sig.is_station {
        ("◉", Color::Cyan)
    } else {
        let lower = sig.display_name.to_lowercase();
        if lower.contains("war") || lower.contains("combat") || lower.contains("conflict") {
            ("⚔", Color::Red)
        } else {
            ("◆", Color::White)
        }
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
