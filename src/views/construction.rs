use crate::event_shim::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::collections::HashSet;

use edcas_common::api::{ConstructionDepotResponse, ConstructionQuery};

use crate::api_client::ApiClient;
use crate::journal_reader::JournalData;
use crate::pins::Pins;
use crate::todo::{ConstructionTodoItem, ConstructionTodoResource, TodoList};
use crate::views::util::FocusArea;
use crate::views::ViewEvent;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

pub struct ConstructionView {
    search_input: String,
    search_active: bool,
    results: Vec<ConstructionDepotResponse>,
    /// Market IDs of all tracked depots — persisted to pins.json.
    tracked_ids: HashSet<i64>,
    /// Tracked depots with full data, sorted by (system_name, station_name).
    depots: Vec<ConstructionDepotResponse>,
    selected_idx: usize,
    scroll_offset: usize,
    detail_scroll: usize,
    focus: FocusArea,
    #[cfg(not(target_arch = "wasm32"))]
    pending_search: Arc<Mutex<Option<Result<Vec<ConstructionDepotResponse>, String>>>>,
    #[cfg(target_arch = "wasm32")]
    pending_search: Rc<RefCell<Option<Vec<ConstructionDepotResponse>>>>,
}

impl ConstructionView {
    pub fn new() -> Self {
        let pins = Pins::load();
        Self {
            search_input: String::new(),
            search_active: false,
            results: Vec::new(),
            tracked_ids: pins.constructions,
            depots: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            detail_scroll: 0,
            focus: FocusArea::List,
            #[cfg(not(target_arch = "wasm32"))]
            pending_search: Arc::new(Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            pending_search: Rc::new(RefCell::new(None)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_search(&mut self) {
        if let Some(result) = self.pending_search.lock().unwrap().take() {
            if let Ok(results) = result {
                self.results = results;
                self.selected_idx = 0;
                self.scroll_offset = 0;
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending_search.borrow_mut().take() {
            self.results = results;
            self.selected_idx = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn on_enter(&mut self, _api: &ApiClient) {}

    fn save_tracked(&self) {
        let mut pins = Pins::load();
        pins.constructions = self.tracked_ids.clone();
        pins.save();
    }

    /// Called whenever journal data updates. Adds newly-discovered depots to the
    /// tracked list without touching the todo list. The user presses `t` to add
    /// a depot to todo manually.
    pub fn update_from_journal(&mut self, api: &ApiClient, journal: &JournalData) {
        let mut any_new = false;

        for depot in journal.construction_depots.values() {
            let mid = depot.submission.market_id;
            let existing = self.depots.iter_mut().find(|d| d.market_id == mid);

            if let Some(entry) = existing {
                // Update with latest journal snapshot.
                entry.progress = depot.submission.progress;
                entry.construction_complete = depot.submission.construction_complete;
                entry.construction_failed = depot.submission.construction_failed;
                entry.resources = depot.submission.resources.iter().map(|r| {
                    edcas_common::api::ConstructionResourceResponse {
                        name: r.name.clone(),
                        display_name: r.display_name.clone(),
                        required_amount: r.required_amount,
                        provided_amount: r.provided_amount,
                        payment: r.payment,
                    }
                }).collect();
            } else {
                // New depot — add to tracked list.
                self.tracked_ids.insert(mid);
                any_new = true;

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let api_owned = api.clone();
                    let submission = depot.submission.clone();
                    api.spawn(async move {
                        let _ = api_owned.submit_construction_depot(&submission).await;
                    });
                }

                self.depots.push(edcas_common::api::ConstructionDepotResponse {
                    market_id: mid,
                    system_address: depot.submission.system_address,
                    station_name: depot.submission.station_name.clone(),
                    system_name: depot.system_name.clone(),
                    progress: depot.submission.progress,
                    construction_complete: depot.submission.construction_complete,
                    construction_failed: depot.submission.construction_failed,
                    last_updated: String::new(),
                    resources: depot.submission.resources.iter().map(|r| {
                        edcas_common::api::ConstructionResourceResponse {
                            name: r.name.clone(),
                            display_name: r.display_name.clone(),
                            required_amount: r.required_amount,
                            provided_amount: r.provided_amount,
                            payment: r.payment,
                        }
                    }).collect(),
                });
            }
        }

        self.depots.sort_by(|a, b| a.system_name.cmp(&b.system_name).then(a.station_name.cmp(&b.station_name)));

        if any_new {
            self.save_tracked();
        }
    }

    fn toggle_todo(&self, journal: &JournalData, todo: &mut TodoList) {
        if let Some(depot) = self.selected_item() {
            let mid = depot.market_id;
            if todo.construction_items.iter().any(|i| i.market_id == mid) {
                todo.remove_construction_item(mid);
            } else {
                let item = if let Some(local) = journal.construction_depots.get(&mid) {
                    construction_todo_item_from_depot(local)
                } else {
                    construction_todo_item_from_response(depot)
                };
                todo.add_construction_item(item);
            }
            todo.save();
        }
    }

    fn remove_selected(&mut self, todo: &mut TodoList) {
        if let Some(depot) = self.selected_item().cloned() {
            let mid = depot.market_id;
            self.tracked_ids.remove(&mid);
            self.depots.retain(|d| d.market_id != mid);
            todo.remove_construction_item(mid);
            todo.save();
            self.save_tracked();
            if self.selected_idx > 0 && self.selected_idx >= self.display_count() {
                self.selected_idx -= 1;
            }
        }
    }

    fn display_count(&self) -> usize {
        let search_deduped = self.results.iter()
            .filter(|r| !self.tracked_ids.contains(&r.market_id))
            .count();
        let tracked = self.depots.len();
        if tracked > 0 && search_deduped > 0 {
            tracked + 1 + search_deduped // separator row
        } else {
            tracked + search_deduped
        }
    }

    fn selected_item(&self) -> Option<&ConstructionDepotResponse> {
        let search: Vec<&ConstructionDepotResponse> = self.results.iter()
            .filter(|r| !self.tracked_ids.contains(&r.market_id))
            .collect();
        let has_sep = !self.depots.is_empty() && !search.is_empty();

        let idx = self.selected_idx;
        if idx < self.depots.len() {
            return self.depots.get(idx);
        }
        let after_tracked = idx - self.depots.len();
        if has_sep {
            if after_tracked == 0 {
                return None; // separator
            }
            search.get(after_tracked - 1).copied()
        } else {
            search.get(after_tracked).copied()
        }
    }

    fn visual_row_of_selected(&self) -> usize {
        let idx = self.selected_idx;
        // 2 header lines: search bar + blank line
        let base = 2usize;

        if idx < self.depots.len() {
            let headers = system_headers_up_to(&self.depots, idx + 1);
            return base + headers + idx;
        }

        let total_headers = system_headers_up_to(&self.depots, self.depots.len());
        let after_tracked = idx - self.depots.len();
        base + total_headers + self.depots.len() + after_tracked
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_input.trim().is_empty() {
            self.results.clear();
            return;
        }
        let pending = Arc::clone(&self.pending_search);
        let api_owned = api.clone();
        let query = ConstructionQuery {
            name: Some(self.search_input.trim().to_string()),
            limit: Some(50),
            ..Default::default()
        };
        api.spawn(async move {
            let result = api_owned.search_construction_depots(&query).await.map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_input.trim().is_empty() {
            self.results.clear();
            return;
        }
        let pending = Rc::clone(&self.pending_search);
        let api = api.clone();
        let query = ConstructionQuery {
            name: Some(self.search_input.trim().to_string()),
            limit: Some(50),
            ..Default::default()
        };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_construction_depots(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData, todo: &mut TodoList) -> ViewEvent {
        if self.search_active {
            match key.code {
                KeyCode::Esc => {
                    self.search_active = false;
                }
                KeyCode::Enter => {
                    self.search_active = false;
                    self.do_search(api);
                }
                KeyCode::Backspace => {
                    self.search_input.pop();
                }
                KeyCode::Char(c) => {
                    self.search_input.push(c);
                }
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Char('f') | KeyCode::Char('/') => {
                self.search_active = true;
                self.search_input.clear();
                self.focus = FocusArea::List;
                return ViewEvent::Consumed;
            }
            KeyCode::Tab | KeyCode::Enter => {
                if matches!(key.modifiers, KeyModifiers::NONE) {
                    match self.focus {
                        FocusArea::List => { self.focus = FocusArea::Detail; self.detail_scroll = 0; }
                        FocusArea::Detail => { self.focus = FocusArea::List; }
                    }
                    return ViewEvent::Consumed;
                }
            }
            KeyCode::Esc | KeyCode::Char('l') => {
                if matches!(self.focus, FocusArea::Detail) {
                    self.focus = FocusArea::List;
                    return ViewEvent::Consumed;
                }
            }
            _ => {}
        }

        match self.focus {
            FocusArea::List => match key.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        if self.selected_item().is_none() && self.selected_idx > 0 {
                            self.selected_idx -= 1;
                        }
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    if self.selected_idx + 1 < self.display_count() {
                        self.selected_idx += 1;
                        if self.selected_item().is_none() && self.selected_idx + 1 < self.display_count() {
                            self.selected_idx += 1;
                        }
                    }
                }
                KeyCode::PageUp => {
                    self.selected_idx = self.selected_idx.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    let max = self.display_count().saturating_sub(1);
                    self.selected_idx = (self.selected_idx + 10).min(max);
                }
                KeyCode::Char('t') => {
                    self.toggle_todo(journal, todo);
                }
                KeyCode::Char('r') | KeyCode::Delete => {
                    // Only remove tracked depots; search results can't be removed (they're not tracked).
                    if self.selected_idx < self.depots.len() {
                        self.remove_selected(todo);
                    }
                }
                _ => {}
            },
            FocusArea::Detail => match key.code {
                KeyCode::Char('w') | KeyCode::Up => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    self.detail_scroll += 1;
                }
                KeyCode::PageUp => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(10);
                }
                KeyCode::PageDown => {
                    self.detail_scroll += 10;
                }
                _ => {}
            },
        }

        ViewEvent::None
    }

    fn auto_scroll(&mut self, visible_height: usize) {
        if visible_height == 0 {
            return;
        }
        let row = self.visual_row_of_selected();
        if row.saturating_sub(2) < self.scroll_offset {
            self.scroll_offset = row.saturating_sub(2);
        } else if row >= self.scroll_offset + visible_height {
            self.scroll_offset = row + 1 - visible_height;
        }
    }

    fn build_list_lines(&self, todo: &TodoList) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let search_bar = if self.search_active {
            format!("Search: {}█", self.search_input)
        } else if self.search_input.is_empty() {
            "Search: (f or / to search)".into()
        } else {
            format!("Search: {}", self.search_input)
        };
        lines.push(Line::from(Span::styled(
            search_bar,
            Style::default().fg(if self.search_active { Color::Yellow } else { Color::DarkGray }),
        )));
        lines.push(Line::from(""));

        if self.depots.is_empty() && self.results.is_empty() {
            lines.push(Line::from(Span::styled(
                "No sites found. Dock at a construction site or search.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        let todo_ids: HashSet<i64> = todo.construction_items.iter().map(|i| i.market_id).collect();
        let search_deduped: Vec<&ConstructionDepotResponse> = self.results.iter()
            .filter(|r| !self.tracked_ids.contains(&r.market_id))
            .collect();
        let has_sep = !self.depots.is_empty() && !search_deduped.is_empty();

        let mut row: usize = 0;

        // Tracked depots — grouped by system name as a tree
        if !self.depots.is_empty() {
            let mut last_sys = "";
            for (logical_idx, depot) in self.depots.iter().enumerate() {
                let sys = if depot.system_name.is_empty() { "Unknown System" } else { depot.system_name.as_str() };
                if sys != last_sys {
                    last_sys = sys;
                    lines.push(Line::from(Span::styled(
                        format!("  {}", sys),
                        Style::default().fg(Color::Cyan),
                    )));
                }
                let is_last_in_group = self.depots.get(logical_idx + 1)
                    .map(|next| {
                        let next_sys = if next.system_name.is_empty() { "Unknown System" } else { next.system_name.as_str() };
                        next_sys != last_sys
                    })
                    .unwrap_or(true);
                let branch = if is_last_in_group { "└" } else { "├" };
                let is_selected = logical_idx == self.selected_idx;
                let in_todo = todo_ids.contains(&depot.market_id);
                lines.push(depot_tree_line(depot, branch, is_selected, in_todo));
                row += 1;
            }
        }

        // Separator
        if has_sep {
            let sep_style = if row == self.selected_idx {
                Style::default().fg(Color::Rgb(255, 200, 100))
            } else {
                Style::default().fg(Color::DarkGray)
            };
            lines.push(Line::from(Span::styled("─── Search ───", sep_style)));
            row += 1;
        }

        // Search results (flat)
        for depot in &search_deduped {
            let is_selected = row == self.selected_idx;
            let in_todo = todo_ids.contains(&depot.market_id);
            lines.push(depot_list_line(depot, is_selected, in_todo));
            row += 1;
        }

        lines
    }

    fn build_detail_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        let selected = self.selected_item();
        let local = selected.and_then(|d| journal.construction_depots.get(&d.market_id));

        if let Some(local) = local {
            build_detail_from_local(&mut lines, local);
        } else if let Some(depot) = selected {
            build_detail_from_response(&mut lines, depot);
        } else {
            lines.push(Line::from(Span::styled(
                "Select a construction site to see details.",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                "Dock at a construction depot to auto-track it.",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData, todo: &TodoList) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(area);

        let list_area = chunks[0];
        let detail_area = chunks[1];

        let visible_height = list_area.height.saturating_sub(2) as usize;
        self.auto_scroll(visible_height);

        let list_focused = matches!(self.focus, FocusArea::List);
        let detail_focused = matches!(self.focus, FocusArea::Detail);

        let list_lines = self.build_list_lines(todo);
        let list_paragraph = Paragraph::new(list_lines)
            .block(
                Block::default()
                    .title(" Construction Sites (f: search | t: todo | r: remove | Enter: details) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(if list_focused {
                        Color::Rgb(255, 140, 0)
                    } else {
                        Color::White
                    })),
            )
            .scroll((self.scroll_offset as u16, 0));
        frame.render_widget(list_paragraph, list_area);

        let detail_lines = self.build_detail_lines(journal);
        let detail_paragraph = Paragraph::new(detail_lines)
            .block(
                Block::default()
                    .title(" Details (w/s: scroll | Esc/l: back) ")
                    .borders(Borders::ALL)
                    .style(Style::default().fg(if detail_focused {
                        Color::Rgb(255, 140, 0)
                    } else {
                        Color::White
                    })),
            )
            .scroll((self.detail_scroll as u16, 0));
        frame.render_widget(detail_paragraph, detail_area);
    }
}

// ── List helpers ─────────────────────────────────────────────────────────────

fn system_headers_up_to(depots: &[ConstructionDepotResponse], up_to: usize) -> usize {
    let mut count = 0usize;
    let mut last: &str = "";
    for depot in depots.iter().take(up_to) {
        let sys = if depot.system_name.is_empty() { "Unknown System" } else { depot.system_name.as_str() };
        if sys != last {
            count += 1;
            last = sys;
        }
    }
    count
}

fn depot_tree_line(
    depot: &ConstructionDepotResponse,
    branch: &str,
    is_selected: bool,
    in_todo: bool,
) -> Line<'static> {
    let name_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let accent_style = if is_selected {
        Style::default().fg(Color::Rgb(255, 200, 100)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Rgb(255, 140, 0))
    };
    let (status_icon, status_color) = depot_status(depot);
    let pct = format!("{:>5.1}%", depot.progress * 100.0);
    let todo_tag = if in_todo { Span::styled(" [T]", Style::default().fg(Color::Green)) } else { Span::raw("") };

    Line::from(vec![
        Span::styled(format!("  {} ", branch), Style::default().fg(Color::DarkGray)),
        Span::styled("★ ", accent_style),
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::styled(depot.station_name.clone(), name_style),
        Span::styled(format!("  {}", pct), progress_color_style(depot.progress)),
        todo_tag,
    ])
}

fn depot_list_line(
    depot: &ConstructionDepotResponse,
    is_selected: bool,
    in_todo: bool,
) -> Line<'static> {
    let name_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };
    let (status_icon, status_color) = depot_status(depot);
    let pct = format!("{:>5.1}%", depot.progress * 100.0);
    let todo_tag = if in_todo { Span::styled(" [T]", Style::default().fg(Color::Green)) } else { Span::raw("") };

    Line::from(vec![
        Span::styled("    ", Style::default()),
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::styled(depot.station_name.clone(), name_style),
        Span::styled(format!("  {}", depot.system_name.clone()), Style::default().fg(Color::DarkGray)),
        Span::styled(format!("  {}", pct), progress_color_style(depot.progress)),
        todo_tag,
    ])
}

fn depot_status(depot: &ConstructionDepotResponse) -> (&'static str, Color) {
    if depot.construction_complete {
        ("✓ ", Color::Green)
    } else if depot.construction_failed {
        ("✗ ", Color::Red)
    } else {
        ("  ", Color::DarkGray)
    }
}

fn progress_color_style(frac: f32) -> Style {
    if frac >= 1.0 {
        Style::default().fg(Color::Green)
    } else if frac >= 0.5 {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Rgb(255, 140, 0))
    }
}

// ── Detail builders ──────────────────────────────────────────────────────────

fn build_detail_from_local(
    lines: &mut Vec<Line<'static>>,
    local: &crate::journal_reader::ConstructionDepotData,
) {
    let s = &local.submission;
    build_depot_header(lines, &s.station_name, s.progress, s.construction_complete, s.construction_failed);

    lines.push(Line::from(""));
    lines.push(section_header(&format!("Resources ({})", s.resources.len())));

    let mut sorted = s.resources.clone();
    sorted.sort_by(|a, b| {
        let done_a = a.provided_amount >= a.required_amount;
        let done_b = b.provided_amount >= b.required_amount;
        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
    });

    for res in &sorted {
        let frac = if res.required_amount == 0 { 1.0f32 } else { res.provided_amount as f32 / res.required_amount as f32 };
        let remaining = (res.required_amount - res.provided_amount).max(0);
        push_resource_line(lines, &res.display_name, res.provided_amount, res.required_amount, frac, remaining, res.payment);
    }
}

fn build_detail_from_response(lines: &mut Vec<Line<'static>>, depot: &ConstructionDepotResponse) {
    build_depot_header(lines, &depot.station_name, depot.progress, depot.construction_complete, depot.construction_failed);

    if !depot.system_name.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  System  ", Style::default().fg(Color::Cyan)),
            Span::styled(depot.system_name.clone(), Style::default().fg(Color::White)),
        ]));
    }
    if !depot.last_updated.is_empty() {
        let ts = depot.last_updated.get(..19).unwrap_or(&depot.last_updated);
        lines.push(Line::from(vec![
            Span::styled("  Updated ", Style::default().fg(Color::Cyan)),
            Span::styled(ts.to_string(), Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(section_header(&format!("Resources ({})", depot.resources.len())));

    let mut sorted = depot.resources.clone();
    sorted.sort_by(|a, b| {
        let done_a = a.provided_amount >= a.required_amount;
        let done_b = b.provided_amount >= b.required_amount;
        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
    });

    for res in &sorted {
        let frac = if res.required_amount == 0 { 1.0f32 } else { res.provided_amount as f32 / res.required_amount as f32 };
        let remaining = (res.required_amount - res.provided_amount).max(0);
        push_resource_line(lines, &res.display_name, res.provided_amount, res.required_amount, frac, remaining, res.payment);
    }
}

fn build_depot_header(
    lines: &mut Vec<Line<'static>>,
    station_name: &str,
    progress: f32,
    complete: bool,
    failed: bool,
) {
    let status = if complete { " ✓ Complete" } else if failed { " ✗ Failed" } else { "" };
    lines.push(Line::from(Span::styled(
        format!("{}{}", station_name, status),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    )));

    let bar_width = 30usize;
    let filled = ((progress * bar_width as f32) as usize).min(bar_width);
    let empty = bar_width - filled;
    let bar = format!("[{}{}] {:>5.1}%", "█".repeat(filled), "░".repeat(empty), progress * 100.0);
    lines.push(Line::from(Span::styled(bar, progress_color_style(progress))));
}

fn push_resource_line(
    lines: &mut Vec<Line<'static>>,
    display_name: &str,
    provided: i32,
    required: i32,
    frac: f32,
    remaining: i32,
    payment: i64,
) {
    let color = if frac >= 1.0 { Color::Green } else if frac > 0.0 { Color::Yellow } else { Color::DarkGray };

    let bar_w = 10usize;
    let filled = ((frac * bar_w as f32) as usize).min(bar_w);
    let empty = bar_w - filled;
    let mini_bar = format!("[{}{}]", "█".repeat(filled), "░".repeat(empty));

    let payment_str = if payment > 0 {
        format!("  {:>7} cr/t", format_thousands(payment))
    } else {
        String::new()
    };

    lines.push(Line::from(vec![
        Span::styled(format!("  {:<36}", display_name), Style::default().fg(Color::White)),
        Span::styled(format!("{:>6}/{:<6}", provided, required), Style::default().fg(color)),
        Span::styled(format!("  {}", mini_bar), Style::default().fg(color)),
        Span::styled(
            format!("  {:>6} left", remaining),
            Style::default().fg(if remaining == 0 { Color::Green } else { Color::DarkGray }),
        ),
        Span::styled(payment_str, Style::default().fg(Color::DarkGray)),
    ]));
}

fn section_header(title: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("─ {} ", title),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    ))
}

fn construction_todo_item_from_depot(
    depot: &crate::journal_reader::ConstructionDepotData,
) -> ConstructionTodoItem {
    let resources = depot.submission.resources.iter()
        .filter(|r| r.provided_amount < r.required_amount)
        .map(|r| ConstructionTodoResource {
            commodity_name: r.name.clone(),
            display_name: r.display_name.clone(),
            required_amount: r.required_amount,
            provided_amount: r.provided_amount,
            payment: r.payment,
        })
        .collect();
    ConstructionTodoItem {
        market_id: depot.submission.market_id,
        station_name: depot.submission.station_name.clone(),
        system_name: depot.system_name.clone(),
        resources,
    }
}

fn construction_todo_item_from_response(depot: &ConstructionDepotResponse) -> ConstructionTodoItem {
    let resources = depot.resources.iter()
        .filter(|r| r.provided_amount < r.required_amount)
        .map(|r| ConstructionTodoResource {
            commodity_name: r.name.clone(),
            display_name: r.display_name.clone(),
            required_amount: r.required_amount,
            provided_amount: r.provided_amount,
            payment: r.payment,
        })
        .collect();
    ConstructionTodoItem {
        market_id: depot.market_id,
        station_name: depot.station_name.clone(),
        system_name: depot.system_name.clone(),
        resources,
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
