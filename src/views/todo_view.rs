use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use std::collections::HashMap;

use crate::engineering_data::{self, MaterialCost};
use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::JournalData;
use crate::todo::{ModKind, TodoList};
use crate::views::util::normalize_commodity_name;
use crate::views::ViewEvent;

enum SelectedKind {
    Engineering(usize),
    Construction(usize),
}

#[derive(Clone, Copy, PartialEq)]
enum TodoFocus {
    Left,
    Right,
}

pub struct TodoView {
    pub todo: TodoList,
    selected_idx: usize,
    scroll_offset: usize,
    show_aggregate: bool,
    focus: TodoFocus,
    resource_selected_idx: usize,
}

impl TodoView {
    pub fn new() -> Self {
        Self {
            todo: TodoList::load(),
            selected_idx: 0,
            scroll_offset: 0,
            show_aggregate: false,
            focus: TodoFocus::Left,
            resource_selected_idx: 0,
        }
    }

    fn total_count(&self) -> usize {
        self.todo.items.len() + self.todo.construction_items.len()
    }

    fn selected_kind(&self) -> Option<SelectedKind> {
        let eng = self.todo.items.len();
        let con = self.todo.construction_items.len();
        if eng + con == 0 {
            return None;
        }
        if self.selected_idx < eng {
            Some(SelectedKind::Engineering(self.selected_idx))
        } else {
            let ci = self.selected_idx - eng;
            if ci < con {
                Some(SelectedKind::Construction(ci))
            } else {
                None
            }
        }
    }

    fn grade_costs(mod_id: &str, grade: u8, kind: &ModKind) -> Vec<MaterialCost> {
        let all_mods = match kind {
            ModKind::Ship => &engineering_data::modifications().ship,
            ModKind::OnFoot => &engineering_data::modifications().onfoot,
        };
        all_mods
            .iter()
            .find(|m| m.id == mod_id)
            .and_then(|m| m.grades.get(&grade.to_string()))
            .cloned()
            .unwrap_or_default()
    }

    fn have_count(mat_name: &str, kind: &ModKind, journal: &JournalData) -> i32 {
        match kind {
            ModKind::Ship => {
                let lower = mat_name.to_lowercase();
                journal
                    .materials_raw
                    .iter()
                    .chain(journal.materials_manufactured.iter())
                    .chain(journal.materials_encoded.iter())
                    .find(|i| i.name.to_lowercase() == lower)
                    .map(|i| i.count)
                    .unwrap_or(0)
            }
            ModKind::OnFoot => {
                let lower = mat_name.to_lowercase();
                journal
                    .shiplocker
                    .items
                    .iter()
                    .chain(journal.shiplocker.components.iter())
                    .chain(journal.shiplocker.consumables.iter())
                    .chain(journal.shiplocker.data.iter())
                    .chain(journal.backpack.items.iter())
                    .chain(journal.backpack.components.iter())
                    .chain(journal.backpack.consumables.iter())
                    .chain(journal.backpack.data.iter())
                    .find(|i| i.localised.to_lowercase() == lower || i.name.to_lowercase() == lower)
                    .map(|i| i.count)
                    .unwrap_or(0)
            }
        }
    }

    fn build_left_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let eng_empty = self.todo.items.is_empty();
        let con_empty = self.todo.construction_items.is_empty();

        if eng_empty && con_empty {
            lines.push(Line::from(Span::styled(
                " No items yet.",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                " Add mods in Engineers tab or dock",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                " at a construction site.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        // Engineering section
        if !eng_empty {
            lines.push(Line::from(Span::styled(
                "─ Engineering ",
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            )));
            for (i, item) in self.todo.items.iter().enumerate() {
                let selected = i == self.selected_idx;
                let kind_tag = match item.kind {
                    ModKind::Ship => "ship",
                    ModKind::OnFoot => "foot",
                };
                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let count_str = if item.count > 1 { format!(" x{}", item.count) } else { String::new() };
                lines.push(Line::from(Span::styled(
                    format!(" {} G{}{}  ({})", item.mod_name, item.grade, count_str, kind_tag),
                    style,
                )));
            }
        }

        // Construction section — grouped by system name
        if !con_empty {
            if !eng_empty {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(Span::styled(
                "─ Construction ",
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            )));

            let offset = self.todo.items.len();

            // Collect groups: (system_name, [(original_idx, site)])
            let mut groups: Vec<(String, Vec<(usize, &crate::todo::ConstructionTodoItem)>)> = Vec::new();
            for (i, site) in self.todo.construction_items.iter().enumerate() {
                let sys = if site.system_name.is_empty() { "Unknown System".to_string() } else { site.system_name.clone() };
                if let Some(g) = groups.iter_mut().find(|(s, _)| s == &sys) {
                    g.1.push((i, site));
                } else {
                    groups.push((sys, vec![(i, site)]));
                }
            }

            for (sys_name, items) in &groups {
                lines.push(Line::from(Span::styled(
                    format!("  {}", sys_name),
                    Style::default().fg(Color::Cyan),
                )));
                let last = items.len().saturating_sub(1);
                for (j, (con_idx, site)) in items.iter().enumerate() {
                    let selected = (offset + con_idx) == self.selected_idx;
                    let branch = if j == last { "└ " } else { "├ " };
                    let remaining = site.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .count();
                    let label = format!("  {}★ {}  ({} needed)", branch, site.station_name, remaining);
                    let style = if selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    lines.push(Line::from(Span::styled(label, style)));
                }
            }
        }

        lines
    }

    fn build_right_lines(&self, journal: &JournalData, ship_cargo: &HashMap<String, i32>, carrier_stock: &HashMap<String, i32>) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        match self.selected_kind() {
            None => {
                lines.push(Line::from("Select an item to see details."));
            }
            Some(SelectedKind::Engineering(idx)) => {
                let item = &self.todo.items[idx];
                let kind_label = match item.kind {
                    ModKind::Ship => "Ship",
                    ModKind::OnFoot => "On-Foot",
                };
                let count_label = if item.count > 1 { format!(" x{}", item.count) } else { String::new() };
                lines.push(Line::from(Span::styled(
                    format!(" {} G{}{}  [{}]", item.mod_name, item.grade, count_label, kind_label),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )));
                lines.push(Line::from(Span::styled(
                    format!("  Module: {}", item.module_type),
                    Style::default().fg(Color::DarkGray),
                )));
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " Materials Required:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(""));

                let costs = Self::grade_costs(&item.mod_id, item.grade, &item.kind);
                if costs.is_empty() {
                    lines.push(Line::from("  No material data found."));
                    return lines;
                }

                let mut all_ok = true;
                for cost in &costs {
                    let have = Self::have_count(&cost.name, &item.kind, journal);
                    let needed = cost.count as i32 * item.count as i32;
                    let ok = have >= needed;
                    if !ok { all_ok = false; }
                    let (bar_color, checkmark) = if ok { (Color::Green, "✓") } else { (Color::Red, "✗") };
                    let bar = progress_bar(have, needed);
                    lines.push(Line::from(vec![
                        Span::styled(format!("  {} {:<32}", checkmark, cost.name), Style::default().fg(bar_color)),
                        Span::styled(format!(" {:>3}/{:<3}", have.min(needed), needed), Style::default().fg(Color::White)),
                        Span::styled(format!(" {}", bar), Style::default().fg(bar_color)),
                    ]));
                }
                lines.push(Line::from(""));
                if all_ok {
                    lines.push(Line::from(Span::styled("  ✓ Ready to engineer!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))));
                } else {
                    let missing = costs.iter()
                        .filter(|c| Self::have_count(&c.name, &item.kind, journal) < c.count as i32 * item.count as i32)
                        .count();
                    lines.push(Line::from(Span::styled(
                        format!("  {} material(s) still needed.", missing),
                        Style::default().fg(Color::Yellow),
                    )));
                }
            }
            Some(SelectedKind::Construction(idx)) => {
                let site = &self.todo.construction_items[idx];

                // Use live journal data if available (currently docked)
                let live = journal.construction_depots.get(&site.market_id);

                lines.push(Line::from(Span::styled(
                    format!(" ★ {}", site.station_name),
                    Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                )));
                if !site.system_name.is_empty() {
                    lines.push(Line::from(Span::styled(
                        format!("  {}", site.system_name),
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                if live.is_some() {
                    lines.push(Line::from(Span::styled(
                        "  (live data)",
                        Style::default().fg(Color::Green),
                    )));
                }
                if self.focus == TodoFocus::Right {
                    lines.push(Line::from(Span::styled(
                        "  Tab: ← panel  f: search nearest",
                        Style::default().fg(Color::DarkGray),
                    )));
                }
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(
                    " Commodities needed:",
                    Style::default().fg(Color::Cyan),
                )));
                lines.push(Line::from(""));

                // Collect resources into a vec so we can index them
                let resources: Vec<(String, String, i32, i32, i64)> = if let Some(depot) = live {
                    depot.submission.resources.iter().map(|r| {
                        (r.name.clone(), r.display_name.clone(), r.provided_amount, r.required_amount, r.payment)
                    }).collect()
                } else {
                    site.resources.iter().map(|r| {
                        (r.commodity_name.clone(), r.display_name.clone(), r.provided_amount, r.required_amount, r.payment)
                    }).collect()
                };

                let dim_orange = Style::default().fg(Color::Rgb(160, 130, 0));
                let mut all_done = true;
                for (res_idx, (raw_name, display_name, provided, required, payment)) in resources.iter().enumerate() {
                    let done = *provided >= *required;
                    if !done { all_done = false; }
                    let remaining = (*required - *provided).max(0);
                    let frac = if *required == 0 { 1.0f32 } else { *provided as f32 / *required as f32 };
                    let base_color = if done { Color::Green } else if frac > 0.0 { Color::Yellow } else { Color::DarkGray };
                    let checkmark = if done { "✓" } else { "·" };
                    let bar_w = 8usize;
                    let filled = ((frac * bar_w as f32) as usize).min(bar_w);
                    let bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(bar_w - filled));

                    let norm = normalize_commodity_name(raw_name);
                    let in_ship = ship_cargo.get(&norm).copied().unwrap_or(0);
                    let in_carrier = carrier_stock.get(&norm).copied().unwrap_or(0);

                    let row_selected = self.focus == TodoFocus::Right && res_idx == self.resource_selected_idx;
                    let row_style = if row_selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(base_color)
                    };

                    let mut spans = vec![
                        Span::styled(
                            format!("  {} {}", checkmark, pad_name(display_name, 34)),
                            if row_selected { row_style } else { Style::default().fg(base_color) },
                        ),
                        Span::styled(
                            format!("{:>7} left", remaining),
                            if row_selected { row_style } else { Style::default().fg(if done { Color::Green } else { Color::DarkGray }) },
                        ),
                        Span::styled(
                            format!("  {}", bar),
                            if row_selected { row_style } else { Style::default().fg(base_color) },
                        ),
                        Span::styled(
                            if *payment > 0 { format!("  {:>7} cr/t", format_cr(*payment)) } else { String::new() },
                            if row_selected { row_style } else { Style::default().fg(Color::DarkGray) },
                        ),
                    ];
                    if !row_selected {
                        if in_ship > 0 {
                            spans.push(Span::styled(format!("  ship:{}", in_ship), dim_orange));
                        }
                        if in_carrier > 0 {
                            spans.push(Span::styled(format!("  carrier:{}", in_carrier), dim_orange));
                        }
                    } else {
                        if in_ship > 0 {
                            spans.push(Span::styled(format!("  ship:{}", in_ship), row_style));
                        }
                        if in_carrier > 0 {
                            spans.push(Span::styled(format!("  carrier:{}", in_carrier), row_style));
                        }
                    }
                    lines.push(Line::from(spans));
                }

                lines.push(Line::from(""));
                if all_done {
                    lines.push(Line::from(Span::styled(
                        "  ✓ All commodities delivered!",
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    )));
                }
            }
        }
        lines
    }

    fn build_aggregate_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        if self.todo.items.is_empty() {
            lines.push(Line::from(Span::styled(
                " No engineering todos.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        // Sum all material costs across all engineering todos.
        // Key: (name, ModKind), value: (needed_total, is_onfoot)
        let mut totals: std::collections::HashMap<String, (i32, crate::todo::ModKind)> =
            std::collections::HashMap::new();
        for item in &self.todo.items {
            for cost in Self::grade_costs(&item.mod_id, item.grade, &item.kind) {
                let entry = totals.entry(cost.name.clone()).or_insert((0, item.kind.clone()));
                entry.0 += cost.count as i32 * item.count as i32;
            }
        }

        lines.push(Line::from(Span::styled(
            format!(" All materials for {} mod(s):", self.todo.items.len()),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));

        // Sort: missing first, then by name.
        let mut sorted: Vec<(String, i32, crate::todo::ModKind)> = totals
            .into_iter()
            .map(|(name, (needed, kind))| (name, needed, kind))
            .collect();
        sorted.sort_by(|a, b| {
            let have_a = Self::have_count(&a.0, &a.2, journal);
            let have_b = Self::have_count(&b.0, &b.2, journal);
            let ok_a = have_a >= a.1;
            let ok_b = have_b >= b.1;
            ok_a.cmp(&ok_b).then(a.0.cmp(&b.0))
        });

        let mut all_ok = true;
        for (name, needed, kind) in &sorted {
            let have = Self::have_count(name, kind, journal);
            let ok = have >= *needed;
            if !ok {
                all_ok = false;
            }
            let (color, check) = if ok { (Color::Green, "✓") } else { (Color::Red, "✗") };
            let bar = progress_bar(have, *needed);
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  {} {:<32}", check, name),
                    Style::default().fg(color),
                ),
                Span::styled(
                    format!(" {:>3}/{:<3}", have.min(*needed), needed),
                    Style::default().fg(Color::White),
                ),
                Span::styled(format!(" {}", bar), Style::default().fg(color)),
            ]));
        }

        lines.push(Line::from(""));
        if all_ok {
            lines.push(Line::from(Span::styled(
                "  ✓ All materials ready!",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            )));
        } else {
            let missing = sorted
                .iter()
                .filter(|(name, needed, kind)| Self::have_count(name, kind, journal) < *needed)
                .count();
            lines.push(Line::from(Span::styled(
                format!("  {} material(s) still needed.", missing),
                Style::default().fg(Color::Yellow),
            )));
        }

        lines
    }

    pub fn handle_key(&mut self, key: &KeyEvent, journal: &JournalData) -> ViewEvent {
        let total = self.total_count();

        // Tab switches focus between left panel and right panel
        if key.code == KeyCode::Tab {
            match self.focus {
                TodoFocus::Left => {
                    if matches!(self.selected_kind(), Some(SelectedKind::Construction(_))) {
                        self.focus = TodoFocus::Right;
                        self.resource_selected_idx = 0;
                    }
                }
                TodoFocus::Right => {
                    self.focus = TodoFocus::Left;
                }
            }
            return ViewEvent::Consumed;
        }

        if self.focus == TodoFocus::Right {
            match key.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    self.resource_selected_idx = self.resource_selected_idx.saturating_sub(1);
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    let resource_count = match self.selected_kind() {
                        Some(SelectedKind::Construction(idx)) => {
                            let site = &self.todo.construction_items[idx];
                            let live = journal.construction_depots.get(&site.market_id);
                            if let Some(depot) = live {
                                depot.submission.resources.len()
                            } else {
                                site.resources.len()
                            }
                        }
                        _ => 0,
                    };
                    if self.resource_selected_idx + 1 < resource_count {
                        self.resource_selected_idx += 1;
                    }
                }
                KeyCode::Char('f') => {
                    if let Some(SelectedKind::Construction(idx)) = self.selected_kind() {
                        let site = &self.todo.construction_items[idx];
                        let live = journal.construction_depots.get(&site.market_id);
                        let commodity_name = if let Some(depot) = live {
                            depot.submission.resources
                                .get(self.resource_selected_idx)
                                .map(|r| normalize_commodity_name(&r.name))
                        } else {
                            site.resources
                                .get(self.resource_selected_idx)
                                .map(|r| r.commodity_name.clone())
                        };
                        if let Some(commodity) = commodity_name {
                            let system = journal.current_system
                                .as_ref()
                                .map(|s| s.name.clone())
                                .unwrap_or_default();
                            return ViewEvent::OpenSearchNearest { commodity, system };
                        }
                    }
                }
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        // Left panel navigation
        match key.code {
            KeyCode::Char('w') | KeyCode::Up => {
                if self.selected_idx > 0 {
                    self.selected_idx -= 1;
                    self.scroll_offset = 0;
                    self.resource_selected_idx = 0;
                }
            }
            KeyCode::Char('s') | KeyCode::Down => {
                if self.selected_idx + 1 < total {
                    self.selected_idx += 1;
                    self.scroll_offset = 0;
                    self.resource_selected_idx = 0;
                }
            }
            KeyCode::PageUp => {
                self.selected_idx = self.selected_idx.saturating_sub(10);
                self.scroll_offset = 0;
                self.resource_selected_idx = 0;
            }
            KeyCode::PageDown => {
                if self.selected_idx + 10 < total {
                    self.selected_idx += 10;
                } else if total > 0 {
                    self.selected_idx = total - 1;
                }
                self.scroll_offset = 0;
                self.resource_selected_idx = 0;
            }
            KeyCode::Char('g') => {
                self.show_aggregate = !self.show_aggregate;
                self.scroll_offset = 0;
            }
            KeyCode::Delete | KeyCode::Char('r') => {
                match self.selected_kind() {
                    Some(SelectedKind::Engineering(idx)) => {
                        let removed = self.todo.decrement_or_remove(idx);
                        if removed && self.selected_idx > 0 && self.selected_idx >= self.total_count() {
                            self.selected_idx -= 1;
                        }
                    }
                    Some(SelectedKind::Construction(idx)) => {
                        let mid = self.todo.construction_items[idx].market_id;
                        self.todo.remove_construction_item(mid);
                        if self.selected_idx > 0 && self.selected_idx >= self.total_count() {
                            self.selected_idx -= 1;
                        }
                    }
                    None => {}
                }
            }
            _ => {}
        }
        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData, carrier_stock: &HashMap<String, i32>) {
        let ship_cargo: HashMap<String, i32> = journal.cargo.iter()
            .map(|item| (normalize_commodity_name(&item.name), item.count))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(46), Constraint::Min(10)])
            .split(area);

        let left_border = if self.focus == TodoFocus::Left { Color::Cyan } else { Color::DarkGray };
        let left_lines = self.build_left_lines();
        frame.render_widget(
            Paragraph::new(left_lines).block(
                Block::default()
                    .title(" Todo (Tab: switch panel  r/Del: remove) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(left_border)),
            ),
            chunks[0],
        );

        let right_border = if self.focus == TodoFocus::Right { Color::Cyan } else { Color::White };
        let (right_lines, right_title) = if self.show_aggregate {
            (
                self.build_aggregate_lines(journal),
                " All Materials (g: item view) ".to_string(),
            )
        } else {
            (
                self.build_right_lines(journal, &ship_cargo, carrier_stock),
                " Materials (w/s: navigate  Tab: switch panel  g: all) ".to_string(),
            )
        };
        let visible = area.height.saturating_sub(2) as usize;
        let max_scroll = right_lines.len().saturating_sub(visible);
        let offset = self.scroll_offset.min(max_scroll) as u16;
        frame.render_widget(
            Paragraph::new(right_lines)
                .block(
                    Block::default()
                        .title(right_title)
                        .borders(Borders::ALL)
                        .style(Style::default().fg(right_border)),
                )
                .scroll((offset, 0)),
            chunks[1],
        );
    }
}

fn format_cr(n: i64) -> String {
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

fn progress_bar(have: i32, need: i32) -> String {
    let ratio = if need <= 0 { 1.0 } else { (have as f64 / need as f64).min(1.0) };
    let filled = (ratio * 12.0).round() as usize;
    let empty = 12usize.saturating_sub(filled);
    format!("[{}{}]", "█".repeat(filled), "░".repeat(empty))
}

/// Returns `s` padded to exactly `width` chars. Truncates with `…` if longer.
fn pad_name(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count > width {
        s.chars().take(width - 1).collect::<String>() + "…"
    } else {
        format!("{:<width$}", s, width = width)
    }
}
