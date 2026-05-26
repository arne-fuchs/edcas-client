use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::event_shim::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tracing::debug;

use crate::journal_reader::{
    BodyComposition as JournalBodyComposition, BodyScan, BodySignal, ConflictData, DiscoveredSignal,
    JournalData, OrganicScan, SaaBodyData, StationData, SystemData, TreeNode, build_body_tree,
};
use crate::settings::Settings;
use crate::views::ViewEvent;

#[derive(PartialEq)]
enum ExplorerFocus {
    SystemInfo,
    Tree,
    Detail,
}

pub struct ExplorerView {
    // Body tree state
    tree: Vec<TreeNode>,
    selected_idx: usize,
    flat_nodes: Vec<FlatNode>,
    scroll_offset: usize,
    system_name: String,
    fss_signals: HashMap<i32, Vec<BodySignal>>,
    saa_data: HashMap<i32, SaaBodyData>,
    stations: Vec<StationData>,
    discovered_signals: Vec<DiscoveredSignal>,
    fss_body_count: Option<u32>,
    fss_non_body_count: Option<u32>,
    fss_all_bodies_found: bool,
    nav_beacon_bodies: Option<u32>,
    organic_scans: Vec<OrganicScan>,
    // System history / info state
    visited_systems: Vec<SystemData>,
    selected_system_idx: usize,
    sys_detail_scroll: usize,
    selected_faction: usize,
    focus: ExplorerFocus,
    // Bodies cached per system address for history navigation
    bodies_cache: HashMap<i64, Vec<BodyScan>>,
}

enum NodeType {
    Body,
    SectionHeader,
    Station(StationData),
    Signal(DiscoveredSignal),
    /// A bio/geo signal attached to a body; FlatNode.body_id is the parent body.
    BodySignal(BodySignal),
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
    radius: f32,    // meters; 0 if body not yet scanned
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
            fss_body_count: None,
            fss_non_body_count: None,
            fss_all_bodies_found: false,
            nav_beacon_bodies: None,
            organic_scans: Vec::new(),
            visited_systems: Vec::new(),
            selected_system_idx: 0,
            sys_detail_scroll: 0,
            selected_faction: 0,
            focus: ExplorerFocus::Tree,
            bodies_cache: HashMap::new(),
        }
    }

    pub fn update(&mut self, journal: &JournalData) {
        // Cache current system bodies so history navigation can show them later.
        if let Some(sys) = &journal.current_system {
            if !journal.bodies.is_empty() {
                self.bodies_cache.insert(sys.system_address, journal.bodies.clone());
            }
        }

        // Update visited systems; reset to current when the active system changes.
        let prev_addr = self.visited_systems.first().map(|s| s.system_address);
        self.visited_systems = journal.visited_systems.clone();
        let new_addr = self.visited_systems.first().map(|s| s.system_address);
        if prev_addr != new_addr {
            self.selected_system_idx = 0;
            self.sys_detail_scroll = 0;
            self.selected_faction = 0;
        }
        if self.selected_system_idx >= self.visited_systems.len() {
            self.selected_system_idx = 0;
        }

        // Build tree for whichever system is currently selected.
        // For the current system, prefer live journal.bodies; if empty (e.g. just jumped back),
        // fall back to cached bodies from a previous visit to this system.
        let current_cached: &[BodyScan] = journal.current_system
            .as_ref()
            .and_then(|s| self.bodies_cache.get(&s.system_address))
            .map(|v| v.as_slice())
            .unwrap_or(&[]);
        let display_bodies: &[BodyScan] = if self.selected_system_idx == 0 {
            if !journal.bodies.is_empty() { &journal.bodies } else { current_cached }
        } else if let Some(sys) = self.visited_systems.get(self.selected_system_idx) {
            self.bodies_cache.get(&sys.system_address).map(|v| v.as_slice()).unwrap_or(&[])
        } else {
            &journal.bodies
        };
        let tree = build_body_tree(display_bodies);
        debug!("Explorer updated with {} bodies, {} tree nodes", display_bodies.len(), tree.len());

        let selected_system_name = self.visited_systems
            .get(self.selected_system_idx)
            .map(|s| s.name.clone())
            .or_else(|| journal.current_system.as_ref().map(|s| s.name.clone()))
            .unwrap_or_default();
        self.system_name = if self.selected_system_idx == 0 {
            journal.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default()
        } else {
            selected_system_name
        };

        self.tree = tree;
        self.fss_signals = journal.fss_signals.clone();
        self.saa_data = journal.saa_data.clone();
        self.stations = journal.stations.clone();
        self.discovered_signals = journal.discovered_signals.clone();
        self.fss_body_count = journal.fss_body_count;
        self.fss_non_body_count = journal.fss_non_body_count;
        self.fss_all_bodies_found = journal.fss_all_bodies_found;
        self.nav_beacon_bodies = journal.nav_beacon_bodies;
        self.organic_scans = journal.organic_scans.clone();
        self.rebuild_flat_nodes();
        if self.selected_idx >= self.flat_nodes.len() {
            self.selected_idx = self.flat_nodes.len().saturating_sub(1);
        }
    }

    fn rebuild_flat_nodes(&mut self) {
        self.flat_nodes.clear();
        let mut placed_stations: HashSet<i64> = HashSet::new();
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
                &self.discovered_signals,
                &self.stations,
                &mut placed_stations,
            );
        }
        let unplaced: Vec<&StationData> = self.stations.iter()
            .filter(|s| !placed_stations.contains(&s.market_id))
            .collect();
        if !unplaced.is_empty() {
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
                radius: 0.0,
            });
            for station in unplaced {
                self.flat_nodes.push(FlatNode {
                    tree_prefix: String::new(),
                    short_name: carrier_display_name(station, &self.discovered_signals),
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
                    radius: 0.0,
                });
            }
        }
        // Only show signals that have no body association in the flat section.
        // Signals with a body_id are already rendered under their body in the tree.
        // Also hide fleet carrier signals for carriers already shown as docked stations.
        let unassociated: Vec<&DiscoveredSignal> = self.discovered_signals.iter()
            .filter(|s| s.body_id.is_none())
            .filter(|s| !is_known_carrier_signal(s, &self.stations))
            .collect();
        if !unassociated.is_empty() {
            self.flat_nodes.push(FlatNode {
                tree_prefix: String::new(),
                short_name: format!("System Signals ({})", unassociated.len()),
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
                radius: 0.0,
            });
            for sig in unassociated {
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
                    radius: 0.0,
                });
            }
        }
    }

    /// Rebuilds the body tree from cached bodies for `selected_system_idx`.
    /// Called after history navigation (a/d) to update the tree without a full journal update.
    fn rebuild_tree_for_selected_system(&mut self) {
        let addr = self.visited_systems
            .get(self.selected_system_idx)
            .map(|s| s.system_address);
        if let Some(addr) = addr {
            self.system_name = self.visited_systems
                .get(self.selected_system_idx)
                .map(|s| s.name.clone())
                .unwrap_or_default();
            let bodies = self.bodies_cache.get(&addr).cloned().unwrap_or_default();
            self.tree = build_body_tree(&bodies);
            self.selected_idx = 0;
            self.scroll_offset = 0;
            self.rebuild_flat_nodes();
        }
    }

    fn get_selected_body(&self) -> Option<&BodyScan> {
        let node = self.flat_nodes.get(self.selected_idx)?;
        // Both Body nodes and BodySignal nodes store a valid body_id
        if !matches!(node.node_type, NodeType::Body | NodeType::BodySignal(_)) {
            return None;
        }
        find_body_in_tree(&self.tree, node.body_id)
    }

    pub fn handle_key(&mut self, key: &KeyEvent) -> ViewEvent {
        match self.focus {
            ExplorerFocus::SystemInfo => match key.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    // Navigate factions upward
                    self.selected_faction = self.selected_faction.saturating_sub(1);
                    ViewEvent::Consumed
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    // Navigate factions downward
                    let faction_count = self.visited_systems
                        .get(self.selected_system_idx)
                        .map(|s| s.factions.len())
                        .unwrap_or(0);
                    if faction_count > 0 {
                        self.selected_faction = (self.selected_faction + 1).min(faction_count - 1);
                    }
                    ViewEvent::Consumed
                }
                KeyCode::Char('a') | KeyCode::Left => {
                    // Browse to older system (higher index = further back in history)
                    if self.selected_system_idx + 1 < self.visited_systems.len() {
                        self.selected_system_idx += 1;
                        self.sys_detail_scroll = 0;
                        self.selected_faction = 0;
                        self.rebuild_tree_for_selected_system();
                    }
                    ViewEvent::Consumed
                }
                KeyCode::Char('d') | KeyCode::Right => {
                    // Browse to newer system (lower index = more recent)
                    if self.selected_system_idx > 0 {
                        self.selected_system_idx -= 1;
                        self.sys_detail_scroll = 0;
                        self.selected_faction = 0;
                        self.rebuild_tree_for_selected_system();
                    }
                    ViewEvent::Consumed
                }
                KeyCode::Tab => {
                    self.focus = ExplorerFocus::Tree;
                    ViewEvent::Consumed
                }
                KeyCode::Char(' ') => {
                    if let Some(system) = self.visited_systems.get(self.selected_system_idx) {
                        if !system.factions.is_empty() {
                            let mut sorted = system.factions.clone();
                            sorted.sort_by(|a, b| {
                                b.influence.partial_cmp(&a.influence).unwrap_or(Ordering::Equal)
                            });
                            let idx = self.selected_faction.min(sorted.len() - 1);
                            return ViewEvent::OpenFactions(sorted[idx].name.clone());
                        }
                    }
                    ViewEvent::None
                }
                _ => ViewEvent::None,
            },
            ExplorerFocus::Tree => {
                let n = self.flat_nodes.len();
                match key.code {
                    KeyCode::Char('w') | KeyCode::Up => {
                        self.selected_idx = self.prev_selectable(self.selected_idx);
                        ViewEvent::Consumed
                    }
                    KeyCode::Char('s') | KeyCode::Down => {
                        self.selected_idx = self.next_selectable(self.selected_idx);
                        ViewEvent::Consumed
                    }
                    KeyCode::PageUp => {
                        let target = self.selected_idx.saturating_sub(10);
                        self.selected_idx = self.nearest_selectable(target);
                        ViewEvent::Consumed
                    }
                    KeyCode::PageDown => {
                        let target = (self.selected_idx + 10).min(n.saturating_sub(1));
                        self.selected_idx = self.nearest_selectable(target);
                        ViewEvent::Consumed
                    }
                    KeyCode::Home => {
                        self.selected_idx = self.nearest_selectable(0);
                        ViewEvent::Consumed
                    }
                    KeyCode::End => {
                        self.selected_idx = self.nearest_selectable(n.saturating_sub(1));
                        ViewEvent::Consumed
                    }
                    KeyCode::Tab | KeyCode::Char('d') | KeyCode::Right => {
                        self.focus = ExplorerFocus::Detail;
                        ViewEvent::Consumed
                    }
                    KeyCode::Char('a') | KeyCode::Left => {
                        self.focus = ExplorerFocus::SystemInfo;
                        ViewEvent::Consumed
                    }
                    _ => ViewEvent::None,
                }
            }
            ExplorerFocus::Detail => match key.code {
                KeyCode::Tab => {
                    self.focus = ExplorerFocus::SystemInfo;
                    ViewEvent::Consumed
                }
                KeyCode::Char('a') | KeyCode::Left | KeyCode::Esc => {
                    self.focus = ExplorerFocus::Tree;
                    ViewEvent::Consumed
                }
                _ => ViewEvent::None,
            },
        }
    }

    // Navigate to the previous non-header node; stay if none exists.
    fn prev_selectable(&self, from: usize) -> usize {
        let mut i = from;
        while i > 0 {
            i -= 1;
            if !matches!(self.flat_nodes[i].node_type, NodeType::SectionHeader) {
                return i;
            }
        }
        from
    }

    // Navigate to the next non-header node; stay if none exists.
    fn next_selectable(&self, from: usize) -> usize {
        let mut i = from + 1;
        while i < self.flat_nodes.len() {
            if !matches!(self.flat_nodes[i].node_type, NodeType::SectionHeader) {
                return i;
            }
            i += 1;
        }
        from
    }

    // Nearest non-header node to `target`: try forward then backward.
    fn nearest_selectable(&self, target: usize) -> usize {
        if self.flat_nodes.is_empty() {
            return 0;
        }
        let target = target.min(self.flat_nodes.len() - 1);
        if !matches!(self.flat_nodes[target].node_type, NodeType::SectionHeader) {
            return target;
        }
        // Forward
        let mut i = target + 1;
        while i < self.flat_nodes.len() {
            if !matches!(self.flat_nodes[i].node_type, NodeType::SectionHeader) {
                return i;
            }
            i += 1;
        }
        // Backward
        let mut i = target;
        while i > 0 {
            i -= 1;
            if !matches!(self.flat_nodes[i].node_type, NodeType::SectionHeader) {
                return i;
            }
        }
        self.selected_idx
    }

    // Returns the 0-based line index in the rendered paragraph for flat_nodes[idx].
    // Line 0 is the system name header. SectionHeader nodes emit an empty line
    // followed by their text line, so nodes after a SectionHeader are further
    // down than their flat_nodes index alone would suggest.
    fn visual_line_of(&self, idx: usize) -> usize {
        let mut line = 1usize; // line 0 is the system name header
        for (i, node) in self.flat_nodes.iter().enumerate() {
            if i == idx {
                break;
            }
            match node.node_type {
                NodeType::SectionHeader => line += 2,
                _ => line += 1,
            }
        }
        line
    }

    fn auto_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        let selected_line = self.visual_line_of(self.selected_idx);
        if selected_line < self.scroll_offset {
            self.scroll_offset = selected_line;
        } else if selected_line >= self.scroll_offset + visible_height {
            self.scroll_offset = selected_line + 1 - visible_height;
        }
    }

    fn build_tree_lines(&self, settings: &Settings) -> Vec<Line<'static>> {
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

        if let Some(total) = self.fss_body_count {
            let non_body = self.fss_non_body_count.unwrap_or(0);
            let (text, style) = if self.fss_all_bodies_found {
                (
                    format!("FSS: All {} found \u{2713}", total),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )
            } else {
                let scanned = self.tree.len() as u32;
                (
                    format!("FSS: {}/{} bodies  (signals: {})", scanned, total, non_body),
                    Style::default().fg(Color::DarkGray),
                )
            };
            lines.push(Line::from(Span::styled(text, style)));
        }

        if let Some(nb) = self.nav_beacon_bodies {
            lines.push(Line::from(Span::styled(
                format!("Nav Beacon: {} bodies", nb),
                Style::default().fg(Color::DarkGray),
            )));
        }

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
                    let dist_str = if node.distance_ls > 0.0 {
                        format!("  {:>8.1} Ls", node.distance_ls)
                    } else {
                        String::new()
                    };
                    let prefix = if node.tree_prefix.is_empty() {
                        "  ".to_string()
                    } else {
                        node.tree_prefix.clone()
                    };
                    if is_selected {
                        lines.push(Line::from(Span::styled(
                            format!("{}◉ {}{}", prefix, node.short_name, dist_str),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Rgb(255, 140, 0))
                                .add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                            Span::styled("◉ ", Style::default().fg(Color::Cyan)),
                            Span::styled(node.short_name.clone(), Style::default().fg(Color::White)),
                            Span::styled(dist_str, Style::default().fg(Color::DarkGray)),
                        ]));
                    }
                }
                NodeType::BodySignal(ref sig) => {
                    let (icon, color) = if sig.is_biological() {
                        settings.icons.get("bio_signal").filter(|i| i.enabled)
                            .map(|i| (i.char.clone(), parse_color(&i.color)))
                            .unwrap_or_else(|| ("🌿".to_string(), Color::Green))
                    } else if sig.is_geological() {
                        settings.icons.get("geo_signal").filter(|i| i.enabled)
                            .map(|i| (i.char.clone(), parse_color(&i.color)))
                            .unwrap_or_else(|| ("🌋".to_string(), Color::Yellow))
                    } else {
                        settings.icons.get("unknown_signal").filter(|i| i.enabled)
                            .map(|i| (i.char.clone(), parse_color(&i.color)))
                            .unwrap_or_else(|| ("◆".to_string(), Color::White))
                    };
                    if is_selected {
                        lines.push(Line::from(Span::styled(
                            format!("{}{} {}", node.tree_prefix, icon, node.short_name),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Rgb(255, 140, 0))
                                .add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled(node.tree_prefix.clone(), Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{icon} "), Style::default().fg(color)),
                            Span::styled(node.short_name.clone(), Style::default().fg(Color::White)),
                        ]));
                    }
                }
                NodeType::Signal(ref sig) => {
                    let (icon, base_color) = signal_icon(sig, settings);
                    let threat_str = sig.threat_level
                        .filter(|&t| t > 0)
                        .map(|t| format!("  ⚠{}", t))
                        .unwrap_or_default();
                    let indent = if node.tree_prefix.is_empty() {
                        "  ".to_string()
                    } else {
                        node.tree_prefix.clone()
                    };
                    if is_selected {
                        lines.push(Line::from(Span::styled(
                            format!("{}{} {}{}", indent, icon, node.short_name, threat_str),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Rgb(255, 140, 0))
                                .add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        let prefix_style = Style::default().fg(Color::DarkGray);
                        lines.push(Line::from(vec![
                            Span::styled(indent, prefix_style),
                            Span::styled(format!("{} ", icon), Style::default().fg(base_color)),
                            Span::styled(node.short_name.clone(), Style::default().fg(Color::White)),
                            Span::styled(threat_str, Style::default().fg(Color::Red)),
                        ]));
                    }
                }
                NodeType::Body => {
                    let (icon, icon_style) = node_icon(node, settings);
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
                    if is_selected {
                        lines.push(Line::from(Span::styled(
                            format!("{}{} {}{}{}", node.tree_prefix, icon, display_name, dist_str, hints),
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::Rgb(255, 140, 0))
                                .add_modifier(Modifier::BOLD),
                        )));
                    } else {
                        let mut spans: Vec<Span<'static>> = vec![
                            Span::styled(node.tree_prefix.clone(), Style::default().fg(Color::DarkGray)),
                            Span::styled(format!("{} ", icon), icon_style),
                            Span::styled(display_name, icon_style),
                            Span::styled(dist_str, Style::default().fg(Color::DarkGray)),
                        ];
                        if !hints.is_empty() {
                            spans.push(Span::styled(hints, Style::default().fg(Color::Cyan)));
                        }
                        lines.push(Line::from(spans));
                    }
                }
            }
        }

        lines
    }

    fn build_body_detail_lines(&self, settings: &Settings, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        if let Some(node) = self.flat_nodes.get(self.selected_idx) {
            if let NodeType::Station(ref station) = node.node_type {
                return build_station_detail(station);
            }
            if let NodeType::Signal(ref sig) = node.node_type {
                return build_signal_detail(sig, settings);
            }
            if matches!(node.node_type, NodeType::SectionHeader) {
                return lines;
            }
            // For a body signal node, show the parent body's full detail
            if matches!(node.node_type, NodeType::BodySignal(_)) {
                // body_id holds the parent body; fall through to body detail rendering below
                // by leaving the node type as-is and letting get_selected_body handle it
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
                let valuable = is_valuable_ring(&ring.ring_class);
                let class_style = if valuable {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let mut spans = vec![
                    Span::styled("  ⌀ ", Style::default().fg(Color::Cyan)),
                    Span::styled(format!("{:<14}", short), Style::default().fg(Color::White)),
                    Span::styled(class, class_style),
                ];
                if valuable {
                    spans.push(Span::styled("  [VALUABLE]", Style::default().fg(Color::Red)));
                }
                lines.push(Line::from(spans));
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
            let inv_map: std::collections::HashMap<&str, i32> = journal.materials_raw.iter()
                .map(|i| (i.name.as_str(), i.count))
                .collect();
            for mat in &mats {
                let name = capitalise(&mat.name);
                let bar = material_bar(mat.percent);
                let inv_count = inv_map.get(mat.name.as_str()).copied().unwrap_or(0);
                let inv_color = if inv_count == 0 {
                    Color::Rgb(255, 140, 0)
                } else if inv_count <= 5 {
                    Color::Yellow
                } else {
                    Color::DarkGray
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<18}", name), Style::default().fg(Color::White)),
                    Span::styled(format!("{:>5.1}%", mat.percent), Style::default().fg(Color::Rgb(255, 140, 0))),
                    Span::styled(format!("  {}", bar), Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("  ×{:>3}", inv_count), Style::default().fg(inv_color)),
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

        let body_organics: Vec<&OrganicScan> = self.organic_scans.iter()
            .filter(|s| s.body_id == Some(body_id))
            .collect();
        if !body_organics.is_empty() {
            lines.push(Line::from(""));
            lines.push(section_header("Biology Scans"));
            for scan in &body_organics {
                let phase_num = match scan.scan_phase.as_str() {
                    "Log" => 1u8,
                    "Sample" => 2,
                    "Analyse" => 3,
                    _ => 0,
                };
                let dots: String = (1u8..=3)
                    .map(|i| if i <= phase_num { '\u{25CF}' } else { '\u{25CB}' })
                    .collect();
                let complete = phase_num >= 3;
                let dot_style = if complete {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("  {:<28}", format!("{} ({})", scan.genus, scan.species)), Style::default().fg(Color::Green)),
                    Span::styled(dots, dot_style),
                ]));
            }
        }

        lines
    }

    fn build_system_info_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let Some(system) = self.visited_systems.get(self.selected_system_idx) else {
            return vec![Line::from(Span::styled(
                "No system data available.",
                Style::default().fg(Color::DarkGray),
            ))];
        };

        let mut lines: Vec<Line<'static>> = Vec::new();

        lines.push(Line::from(Span::styled(
            system.name.clone(),
            Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        lines.push(sys_section("Overview"));
        sys_row(&mut lines, "Economy", system.economy.clone());
        if !system.second_economy.is_empty() {
            sys_row(&mut lines, "2nd Economy", system.second_economy.clone());
        }
        sys_row(&mut lines, "Government", system.government.clone());
        sys_row(&mut lines, "Allegiance", system.allegiance.clone());
        sys_row(&mut lines, "Security", system.security.clone());
        sys_row(&mut lines, "Population", format_population(system.population));
        lines.push(Line::from(""));

        let has_power = system.controlling_power.as_ref().map(|p| !p.is_empty()).unwrap_or(false)
            || !system.powers.is_empty();
        if has_power {
            lines.push(sys_section("Powerplay"));
            if let Some(ref power) = system.controlling_power {
                if !power.is_empty() {
                    sys_row(&mut lines, "Control", power.clone());
                }
            }
            if !system.powers.is_empty() {
                sys_row(&mut lines, "Powers", system.powers.join(", "));
            }
            lines.push(Line::from(""));
        }

        // Colonisation — current system only
        if self.selected_system_idx == 0 {
            let depots_in_system: Vec<_> = journal.construction_depots.values()
                .filter(|d| d.system_name == system.name)
                .collect();
            let is_architect = journal.claimed_systems.contains_key(&system.system_address);

            if is_architect || !depots_in_system.is_empty() {
                lines.push(sys_section("Colonisation"));
                if is_architect {
                    let cmdr = if journal.pilot.name.is_empty() {
                        "You".to_string()
                    } else {
                        format!("CMDR {}", journal.pilot.name)
                    };
                    lines.push(Line::from(Span::styled(
                        format!("  Architect: {}", cmdr),
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                    )));
                }
                for depot in &depots_in_system {
                    let pct = depot.submission.progress * 100.0;
                    let filled = (depot.submission.progress * 16.0).round() as usize;
                    let bar_color = if pct >= 100.0 { Color::Green } else if pct > 50.0 { Color::Yellow } else { Color::Cyan };
                    let remaining = depot.submission.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .count();
                    lines.push(Line::from(Span::styled(
                        format!("  \u{2605} {}", depot.submission.station_name),
                        Style::default().fg(Color::White),
                    )));
                    lines.push(Line::from(vec![
                        Span::raw(format!("  {:>5.1}% [", pct)),
                        Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
                        Span::styled("\u{2591}".repeat(16 - filled), Style::default().fg(Color::DarkGray)),
                        Span::raw(format!("] {}×", remaining)),
                    ]));
                }
                lines.push(Line::from(""));
            }
        }

        if !system.factions.is_empty() {
            let mut sorted = system.factions.clone();
            sorted.sort_by(|a, b| b.influence.partial_cmp(&a.influence).unwrap_or(Ordering::Equal));

            lines.push(sys_section(&format!("Factions ({}) — space: open", sorted.len())));

            let sel = self.selected_faction.min(sorted.len().saturating_sub(1));

            for (i, faction) in sorted.iter().enumerate() {
                let is_sel = i == sel && self.focus == ExplorerFocus::SystemInfo;
                let is_controlling = faction.name == system.system_faction;

                let name_style = if is_sel {
                    Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
                } else if is_controlling {
                    Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                };

                let tag = if is_controlling { " \u{2605}" } else { "" };
                lines.push(Line::from(Span::styled(
                    format!("  {}{}", faction.name, tag),
                    name_style,
                )));

                let pct = faction.influence * 100.0;
                let filled = (faction.influence * 16.0).round() as usize;
                let bar_color = if pct < 15.0 { Color::Red } else if pct < 40.0 { Color::Yellow } else { Color::Green };
                lines.push(Line::from(vec![
                    Span::raw(format!("  {:>5.1}% [", pct)),
                    Span::styled("\u{2588}".repeat(filled), Style::default().fg(bar_color)),
                    Span::styled("\u{2591}".repeat(16 - filled), Style::default().fg(Color::DarkGray)),
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
                        format!("  {}", state_parts.join(" | ")),
                        Style::default().fg(Color::DarkGray),
                    )));
                }

                if let Some(ref c) = faction.conflict {
                    for l in sys_conflict_lines(c) {
                        lines.push(l);
                    }
                }

                lines.push(Line::from(""));
            }
        }

        lines
    }

    // Build history bar spans. inner_width is the usable width (area width - 2 for borders).
    fn build_history_spans(&self, inner_width: usize) -> Vec<Span<'static>> {
        let n = self.visited_systems.len();
        if n == 0 {
            return vec![Span::styled(
                "No systems visited yet.",
                Style::default().fg(Color::DarkGray),
            )];
        }

        let arrow = " \u{2192} "; // " → "
        let arrow_w = 3usize;
        let more_w = 2usize; // "◄ "

        // visited_systems[0] = newest/current, [n-1] = oldest
        // Display order: oldest (display index 0) → newest (display index n-1)
        // sel in display space = n - 1 - selected_system_idx
        let sel_disp = n - 1 - self.selected_system_idx.min(n - 1);

        let name_widths: Vec<usize> = (0..n)
            .map(|disp| self.visited_systems[n - 1 - disp].name.chars().count())
            .collect();

        // Start window at sel_disp, expand outward to fill inner_width.
        let mut lo = sel_disp;
        let mut hi = sel_disp;
        let mut used = name_widths[sel_disp];

        loop {
            let mut expanded = false;

            // Expand right (toward newer / lower visited_systems index)
            if hi + 1 < n {
                let needed = arrow_w + name_widths[hi + 1];
                let left_ind = if lo > 0 { more_w } else { 0 };
                if used + needed + left_ind <= inner_width {
                    used += needed;
                    hi += 1;
                    expanded = true;
                }
            }

            // Expand left (toward older / higher visited_systems index)
            if lo > 0 {
                let needed = arrow_w + name_widths[lo - 1];
                let left_ind = if lo - 1 > 0 { more_w } else { 0 };
                if used + needed + left_ind <= inner_width {
                    used += needed;
                    lo -= 1;
                    expanded = true;
                }
            }

            if !expanded {
                break;
            }
        }

        let has_more_left = lo > 0;

        let mut spans: Vec<Span<'static>> = Vec::new();

        if has_more_left {
            spans.push(Span::styled("\u{25C4} ", Style::default().fg(Color::DarkGray)));
        }

        for disp_idx in lo..=hi {
            if disp_idx > lo {
                spans.push(Span::styled(arrow.to_string(), Style::default().fg(Color::DarkGray)));
            }

            let sys_idx = n - 1 - disp_idx; // map display → visited_systems index
            let sys = &self.visited_systems[sys_idx];
            let is_selected = sys_idx == self.selected_system_idx;
            let is_current = sys_idx == 0;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD)
            } else if is_current {
                Style::default().fg(Color::Rgb(255, 140, 0))
            } else {
                Style::default().fg(Color::White)
            };

            spans.push(Span::styled(sys.name.clone(), style));
        }

        spans
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, settings: &Settings, journal: &JournalData) {
        // Split: history bar (3 rows) + main panels
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)])
            .split(area);

        let history_area = rows[0];
        let main_area = rows[1];

        // Render history bar
        self.render_history_bar(frame, history_area);

        // Split main area: system info | nav tree | body detail
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(40),
                Constraint::Percentage(35),
            ])
            .split(main_area);

        let sys_area = cols[0];
        let tree_area = cols[1];
        let detail_area = cols[2];

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        // System info panel
        let sys_border_style = if self.focus == ExplorerFocus::SystemInfo {
            active_border
        } else {
            inactive_border
        };
        let sys_info_lines = self.build_system_info_lines(journal);
        let sys_visible = sys_area.height.saturating_sub(2) as usize;
        let sys_content_len = sys_info_lines.len();
        let sys_scroll = self.sys_detail_scroll.min(sys_content_len.saturating_sub(sys_visible));
        frame.render_widget(
            Paragraph::new(sys_info_lines)
                .block(
                    Block::default()
                        .title(" System (w/s: factions  a/d: history  space: open) ")
                        .borders(Borders::ALL)
                        .border_style(sys_border_style),
                )
                .scroll((sys_scroll as u16, 0)),
            sys_area,
        );

        // Navigation tree panel
        let tree_border_style = if self.focus == ExplorerFocus::Tree {
            active_border
        } else {
            inactive_border
        };
        let visible_height = tree_area.height.saturating_sub(2) as usize;
        self.auto_scroll(visible_height);
        let tree_lines = self.build_tree_lines(settings);
        frame.render_widget(
            Paragraph::new(tree_lines)
                .block(
                    Block::default()
                        .title(" Navigation (w/s ↑↓  PgUp/PgDn) ")
                        .borders(Borders::ALL)
                        .border_style(tree_border_style),
                )
                .scroll((self.scroll_offset as u16, 0)),
            tree_area,
        );

        // Body detail panel
        let detail_border_style = if self.focus == ExplorerFocus::Detail {
            active_border
        } else {
            inactive_border
        };
        let detail_lines = self.build_body_detail_lines(settings, journal);
        frame.render_widget(
            Paragraph::new(detail_lines)
                .block(
                    Block::default()
                        .title(" Body Details ")
                        .borders(Borders::ALL)
                        .border_style(detail_border_style),
                ),
            detail_area,
        );
    }

    fn render_history_bar(&self, frame: &mut Frame, area: Rect) {
        let inner_width = area.width.saturating_sub(2) as usize;
        let is_active = self.focus == ExplorerFocus::SystemInfo;
        let border_style = if is_active {
            Style::default().fg(Color::Rgb(255, 140, 0))
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let spans = self.build_history_spans(inner_width);
        frame.render_widget(
            Paragraph::new(Line::from(spans))
                .block(
                    Block::default()
                        .title(" Jump History ")
                        .borders(Borders::ALL)
                        .border_style(border_style),
                ),
            area,
        );
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
    discovered_signals: &[DiscoveredSignal],
    stations: &[StationData],
    placed_stations: &mut HashSet<i64>,
) {
    let connector = if is_last { "└─ " } else { "├─ " };
    let continuation = if is_last { "   " } else { "│  " };
    let tree_prefix = format!("{}{}", parent_prefix, connector);
    let child_prefix = format!("{}{}", parent_prefix, continuation);

    let body = node.data.as_ref();
    let is_barycentre = node.name.to_ascii_lowercase().contains("barycentre");
    let is_star = body.map(|b| !b.star_type.is_empty()).unwrap_or(false);
    let short_name = strip_system_prefix(&node.name, system_name);
    let body_dist = body.map(|b| b.distance_from_arrival_ls).unwrap_or(0.0);

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
        distance_ls: body_dist,
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
        radius: body.map(|b| b.radius).unwrap_or(0.0),
    });

    // Find stations orbiting this body.
    // Primary: exact host_body_id match (from SupercruiseExit journal event).
    // Fallback: distance proximity, but only for stations without a known host_body_id and
    // only on non-star, non-barycentre bodies with a known distance.
    let orbiting: Vec<&StationData> = if !is_star && !is_barycentre {
        stations.iter()
            .filter(|s| {
                if placed_stations.contains(&s.market_id) { return false; }
                if let Some(hb) = s.host_body_id {
                    hb == node.body_id
                } else {
                    // Distance fallback only when body distance is known and tight (≤2 ls or 0.5%)
                    body_dist > 0.0 && s.dist_from_star_ls > 0.0 && {
                        let diff = (s.dist_from_star_ls - body_dist).abs();
                        diff < (body_dist * 0.005).max(2.0)
                    }
                }
            })
            .collect()
    } else {
        Vec::new()
    };
    // Mark placed BEFORE recursing so children don't pick up these stations.
    for s in &orbiting {
        placed_stations.insert(s.market_id);
    }
    let n_orbiting = orbiting.len();

    // Collect signals for this body (prefer SAA over FSS)
    let body_signals: Vec<BodySignal> = if let Some(saa) = saa_data.get(&node.body_id) {
        saa.signals.clone()
    } else if let Some(sigs) = fss_signals.get(&node.body_id) {
        sigs.clone()
    } else {
        Vec::new()
    };

    let n_children = node.children.len();
    let n_sigs = body_signals.len();
    let disc_sigs: Vec<&DiscoveredSignal> = discovered_signals
        .iter()
        .filter(|s| s.body_id == Some(node.body_id))
        .collect();
    let n_disc = disc_sigs.len();

    // Orbital children
    for (i, child) in node.children.iter().enumerate() {
        let is_last = i == n_children - 1 && n_sigs == 0 && n_disc == 0 && n_orbiting == 0;
        flatten_node(child, &child_prefix, is_last, result, system_name, fss_signals, saa_data, discovered_signals, stations, placed_stations);
    }

    // Bio/geo signal children
    for (j, sig) in body_signals.iter().enumerate() {
        let is_last = j == n_sigs - 1 && n_disc == 0 && n_orbiting == 0;
        let sig_connector = if is_last { "└─ " } else { "├─ " };
        let sig_prefix = format!("{}{}", child_prefix, sig_connector);
        let label = format!("{} ×{}", sig.display_type(), sig.count);
        result.push(FlatNode {
            tree_prefix: sig_prefix,
            short_name: label,
            body_id: node.body_id,
            distance_ls: 0.0,
            has_rings: false,
            landable: false,
            planet_class: String::new(),
            star_type: String::new(),
            is_barycentre: false,
            composition: None,
            bio_signal_count: 0,
            geo_signal_count: 0,
            node_type: NodeType::BodySignal(sig.clone()),
            radius: 0.0,
        });
    }

    // Body-associated system signals (FssSignalDiscovered with BodyID)
    for (k, sig) in disc_sigs.iter().enumerate() {
        let is_last = k == n_disc - 1 && n_orbiting == 0;
        let sig_connector = if is_last { "└─ " } else { "├─ " };
        let sig_prefix = format!("{}{}", child_prefix, sig_connector);
        result.push(FlatNode {
            tree_prefix: sig_prefix,
            short_name: sig.display_name.clone(),
            body_id: node.body_id,
            distance_ls: 0.0,
            has_rings: false,
            landable: false,
            planet_class: String::new(),
            star_type: String::new(),
            is_barycentre: false,
            composition: None,
            bio_signal_count: 0,
            geo_signal_count: 0,
            node_type: NodeType::Signal((*sig).clone()),
            radius: 0.0,
        });
    }

    // Orbiting stations as children of this body
    for (i, station) in orbiting.iter().enumerate() {
        let conn = if i == n_orbiting - 1 { "└─ " } else { "├─ " };
        let prefix = format!("{}{}", child_prefix, conn);
        result.push(FlatNode {
            tree_prefix: prefix,
            short_name: carrier_display_name(station, discovered_signals),
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
            node_type: NodeType::Station((*station).clone()),
            radius: 0.0,
        });
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
    if !station.allegiance.is_empty() {
        detail_row(&mut lines, "Allegiance", station.allegiance.clone());
    }
    if !station.faction.is_empty() {
        detail_row(&mut lines, "Faction", station.faction.clone());
    }
    if !station.government.is_empty() {
        detail_row(&mut lines, "Government", station.government.clone());
    }
    if !station.economy.is_empty() {
        let econ = if station.secondary_economies.is_empty() {
            station.economy.clone()
        } else {
            let secs: Vec<String> = station.secondary_economies.iter()
                .map(|(name, prop)| format!("{} ({:.0}%)", name, prop * 100.0))
                .collect();
            format!("{} / {}", station.economy, secs.join(", "))
        };
        detail_row(&mut lines, "Economy", econ);
    }
    if let Some((s, m, l)) = station.landing_pads {
        detail_row(&mut lines, "Landing Pads", format!("S:{s}  M:{m}  L:{l}"));
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

fn build_signal_detail(sig: &DiscoveredSignal, settings: &Settings) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    let (icon, color) = signal_icon(sig, settings);
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

fn parse_color(s: &str) -> Color {
    match s.to_lowercase().as_str() {
        "white"        => Color::White,
        "black"        => Color::Black,
        "red"          => Color::Red,
        "green"        => Color::Green,
        "blue"         => Color::Blue,
        "yellow"       => Color::Yellow,
        "cyan"         => Color::Cyan,
        "magenta"      => Color::Magenta,
        "gray" | "grey"             => Color::Gray,
        "darkgray" | "darkgrey"     => Color::DarkGray,
        "lightred"     => Color::LightRed,
        "lightgreen"   => Color::LightGreen,
        "lightyellow"  => Color::LightYellow,
        "lightblue"    => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan"    => Color::LightCyan,
        s if s.starts_with('#') && s.len() == 7 => {
            let r = u8::from_str_radix(&s[1..3], 16).unwrap_or(255);
            let g = u8::from_str_radix(&s[3..5], 16).unwrap_or(255);
            let b = u8::from_str_radix(&s[5..7], 16).unwrap_or(255);
            Color::Rgb(r, g, b)
        }
        _ => Color::White,
    }
}

fn size_planet_icon(radius: f32, is_gas: bool) -> &'static str {
    if is_gas {
        if radius < 20_000_000.0 { "∘" }
        else if radius < 60_000_000.0 { "○" }
        else { "◯" }
    } else {
        if radius < 1_000_000.0 { "·" }
        else if radius < 4_000_000.0 { "•" }
        else if radius < 9_000_000.0 { "●" }
        else { "⬤" }
    }
}

fn size_star_icon(radius: f32) -> &'static str {
    if radius < 20_000_000.0 { "·" }            // neutron stars, white dwarfs
    else if radius < 300_000_000.0 { "✦" }       // brown dwarfs, tiny red dwarfs
    else if radius < 2_000_000_000.0 { "★" }     // M, K, G, F main sequence
    else if radius < 10_000_000_000.0 { "✸" }    // A, B, O and large stars
    else { "✵" }                                  // giants and supergiants
}

fn node_icon(node: &FlatNode, settings: &Settings) -> (String, Style) {
    if node.planet_class.is_empty() {
        if node.is_barycentre {
            return ("⊕".to_string(), Style::default().fg(Color::DarkGray));
        }
        if node.short_name.to_ascii_lowercase().contains("belt cluster") {
            return settings.icons.get("belt_cluster").filter(|i| i.enabled)
                .map(|i| (i.char.clone(), Style::default().fg(parse_color(&i.color))))
                .unwrap_or_else(|| ("◆".to_string(), Style::default().fg(Color::Rgb(160, 120, 80))));
        }
        // Star: color from settings, char from size tier (or settings/hardcoded fallback)
        let color = settings.stars.get(&node.star_type).filter(|i| i.enabled)
            .map(|i| parse_color(&i.color))
            .unwrap_or(Color::Rgb(255, 140, 0));
        let icon = if node.radius > 0.0 {
            size_star_icon(node.radius).to_string()
        } else {
            settings.stars.get(&node.star_type).filter(|i| i.enabled)
                .map(|i| i.char.clone())
                .unwrap_or_else(|| "★".to_string())
        };
        (icon, Style::default().fg(color))
    } else {
        body_class_icon(&node.planet_class, node.radius, settings)
    }
}

fn body_class_icon(planet_class: &str, radius: f32, settings: &Settings) -> (String, Style) {
    let lower = planet_class.to_lowercase();
    let is_gas = lower.contains("gas") || lower.contains("sudarsky") || lower.contains("giant");

    // Color: settings → hardcoded class default
    let (color, bold) = settings.planets.get(planet_class).filter(|i| i.enabled)
        .map(|i| (parse_color(&i.color), false))
        .unwrap_or_else(|| {
            if lower.contains("earthlike")                                          { (Color::Green, true) }
            else if lower.contains("water world") || lower.contains("water giant") { (Color::Blue, false) }
            else if lower.contains("ammonia")                                       { (Color::LightMagenta, false) }
            else if lower.contains("metal rich")                                    { (Color::Red, false) }
            else if lower.contains("high metal")                                    { (Color::LightRed, false) }
            else if lower.contains("icy")                                           { (Color::Cyan, false) }
            else if lower.contains("rocky")                                         { (Color::Gray, false) }
            else if is_gas                                                           { (Color::Rgb(255, 165, 0), false) }
            else                                                                    { (Color::White, false) }
        });

    // Char: size-based (when scanned) → settings → hardcoded
    let icon = if radius > 0.0 {
        size_planet_icon(radius, is_gas).to_string()
    } else {
        settings.planets.get(planet_class).filter(|i| i.enabled)
            .map(|i| i.char.clone())
            .unwrap_or_else(|| if is_gas { "○" } else { "●" }.to_string())
    };

    let style = if bold { Style::default().fg(color).add_modifier(Modifier::BOLD) }
                else    { Style::default().fg(color) };
    (icon, style)
}

fn signal_icon(sig: &DiscoveredSignal, settings: &Settings) -> (String, Color) {
    if sig.uss_type.is_some() {
        let threat = sig.threat_level.unwrap_or(0);
        if threat >= 4 {
            ("⚠".to_string(), Color::Red)
        } else if threat >= 2 {
            ("⚠".to_string(), Color::Rgb(255, 140, 0))
        } else {
            ("⚠".to_string(), Color::Yellow)
        }
    } else if sig.is_station {
        ("◉".to_string(), Color::Cyan)
    } else {
        let lower = sig.display_name.to_lowercase();
        if lower.contains("war") || lower.contains("combat") || lower.contains("conflict") {
            settings.icons.get("human_signal").filter(|i| i.enabled)
                .map(|i| (i.char.clone(), parse_color(&i.color)))
                .unwrap_or_else(|| ("⚔".to_string(), Color::Red))
        } else {
            settings.icons.get("unknown_signal").filter(|i| i.enabled)
                .map(|i| (i.char.clone(), parse_color(&i.color)))
                .unwrap_or_else(|| ("◆".to_string(), Color::White))
        }
    }
}

// ── System info helpers ──────────────────────────────────────────────────────

fn sys_section(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("─ {} ", title),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    ))
}

fn sys_row(lines: &mut Vec<Line<'static>>, label: &str, value: String) {
    lines.push(Line::from(vec![
        Span::styled(format!("  {:<12}", label), Style::default().fg(Color::DarkGray)),
        Span::styled(value, Style::default().fg(Color::White)),
    ]));
}

fn sys_conflict_lines(c: &ConflictData) -> Vec<Line<'static>> {
    let type_label = match c.war_type.to_lowercase().as_str() {
        "war" => "War",
        "civilwar" => "Civil War",
        "election" => "Election",
        _ => "Conflict",
    };
    let status = if c.status.is_empty() { "active".to_string() } else { c.status.clone() };
    let score_color = Color::Rgb(255, 140, 0);
    vec![
        Line::from(vec![
            Span::styled("  ".to_string(), Style::default()),
            Span::styled(format!("{} ({})", type_label, status), Style::default().fg(score_color)),
            Span::styled("  vs  ".to_string(), Style::default().fg(Color::DarkGray)),
            Span::styled(c.opponent.clone(), Style::default().fg(Color::Yellow)),
        ]),
        Line::from(Span::styled(
            format!("  Score: {} \u{2013} {}", c.our_won_days, c.opponent_won_days),
            Style::default().fg(score_color),
        )),
    ]
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

fn material_bar(percent: f64) -> String {
    let filled = (percent / 10.0).round() as usize;
    let empty = 10usize.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled.min(10)), "░".repeat(empty))
}

fn is_valuable_ring(ring_class: &str) -> bool {
    let lower = ring_class.to_lowercase();
    lower.contains("metal") || lower.contains("rocky")
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

/// Returns true if this discovered signal represents a fleet carrier already
/// tracked as a docked station, so it can be filtered from the signals list.
fn is_known_carrier_signal(sig: &DiscoveredSignal, stations: &[StationData]) -> bool {
    if sig.signal_type.as_deref() != Some("FleetCarrier") {
        return false;
    }
    stations.iter().any(|s| {
        s.station_type == "FleetCarrier"
            && (sig.display_name == s.name || sig.display_name.ends_with(&s.name))
    })
}

/// Returns the enriched display name for a fleet carrier station by looking up
/// the matching FSS signal (which includes the full owner+callsign name).
fn carrier_display_name(station: &StationData, signals: &[DiscoveredSignal]) -> String {
    if station.station_type != "FleetCarrier" {
        return station.name.clone();
    }
    signals
        .iter()
        .filter(|s| s.signal_type.as_deref() == Some("FleetCarrier"))
        .find(|s| s.display_name == station.name || s.display_name.ends_with(&station.name))
        .map(|s| s.display_name.clone())
        .unwrap_or_else(|| station.name.clone())
}
