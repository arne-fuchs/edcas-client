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
use crate::views::ViewEvent;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

enum FocusArea {
    List,
    Detail,
}

pub struct ConstructionView {
    search_input: String,
    search_active: bool,
    results: Vec<ConstructionDepotResponse>,
    pinned_ids: HashSet<i64>,
    pinned_results: Vec<ConstructionDepotResponse>,
    selected_idx: usize,
    scroll_offset: usize,
    detail_scroll: usize,
    focus: FocusArea,
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
            pinned_ids: pins.constructions,
            pinned_results: Vec::new(),
            selected_idx: 0,
            scroll_offset: 0,
            detail_scroll: 0,
            focus: FocusArea::List,
            #[cfg(target_arch = "wasm32")]
            pending_search: Rc::new(RefCell::new(None)),
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

    pub fn on_enter(&mut self, api: &ApiClient) {
        #[cfg(not(target_arch = "wasm32"))]
        self.refresh_pins(api);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_pins(&mut self, api: &ApiClient) {
        self.pinned_results.clear();
        for &mid in &self.pinned_ids.clone() {
            let query = ConstructionQuery {
                market_id: Some(mid),
                ..Default::default()
            };
            if let Ok(mut res) = api.search_construction_depots(&query) {
                if let Some(depot) = res.pop() {
                    self.pinned_results.push(depot);
                }
            }
        }
        self.pinned_results.sort_by(|a, b| a.station_name.cmp(&b.station_name));
    }

    fn save_pins(&self) {
        let mut pins = Pins::load();
        pins.constructions = self.pinned_ids.clone();
        pins.save();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn toggle_pin(&mut self, api: &ApiClient) {
        if let Some(depot) = self.selected_item() {
            let mid = depot.market_id;
            if self.pinned_ids.contains(&mid) {
                self.pinned_ids.remove(&mid);
                self.pinned_results.retain(|d| d.market_id != mid);
            } else {
                self.pinned_ids.insert(mid);
                let query = ConstructionQuery { market_id: Some(mid), ..Default::default() };
                if let Ok(mut res) = api.search_construction_depots(&query) {
                    if let Some(d) = res.pop() {
                        self.pinned_results.push(d);
                        self.pinned_results.sort_by(|a, b| a.station_name.cmp(&b.station_name));
                    }
                }
            }
            self.save_pins();
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn toggle_pin(&mut self, _api: &ApiClient) {
        if let Some(depot) = self.selected_item() {
            let mid = depot.market_id;
            if self.pinned_ids.contains(&mid) {
                self.pinned_ids.remove(&mid);
                self.pinned_results.retain(|d| d.market_id != mid);
            } else {
                self.pinned_ids.insert(mid);
            }
            self.save_pins();
        }
    }

    /// Pin a depot from local journal data (when docked at one).
    pub fn pin_local_depot(
        &mut self,
        api: &ApiClient,
        journal: &JournalData,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            for depot in journal.construction_depots.values() {
                let _ = api.submit_construction_depot(&depot.submission);
            }
        }
        for depot in journal.construction_depots.values() {
            let mid = depot.submission.market_id;
            if !self.pinned_ids.contains(&mid) {
                self.pinned_ids.insert(mid);
            }
        }
        self.save_pins();
        #[cfg(not(target_arch = "wasm32"))]
        self.refresh_pins(api);
    }

    fn display_count(&self) -> usize {
        let search_deduped = self
            .results
            .iter()
            .filter(|r| !self.pinned_ids.contains(&r.market_id))
            .count();
        let pinned_count = self.pinned_results.len();
        if pinned_count > 0 && search_deduped > 0 {
            pinned_count + 1 + search_deduped // separator row
        } else {
            pinned_count + search_deduped
        }
    }

    fn selected_item(&self) -> Option<&ConstructionDepotResponse> {
        let pinned = &self.pinned_results;
        let search: Vec<&ConstructionDepotResponse> = self
            .results
            .iter()
            .filter(|r| !self.pinned_ids.contains(&r.market_id))
            .collect();
        let has_sep = !pinned.is_empty() && !search.is_empty();

        let idx = self.selected_idx;
        if idx < pinned.len() {
            return pinned.get(idx);
        }
        let after_pinned = idx - pinned.len();
        if has_sep {
            if after_pinned == 0 {
                return None; // separator
            }
            search.get(after_pinned - 1).copied()
        } else {
            search.get(after_pinned).copied()
        }
    }

    fn visual_row_of_selected(&self) -> usize {
        self.selected_idx
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_input.trim().is_empty() && self.pinned_ids.is_empty() {
            self.results.clear();
            return;
        }
        let query = ConstructionQuery {
            name: if self.search_input.trim().is_empty() { None } else { Some(self.search_input.trim().to_string()) },
            limit: Some(50),
            ..Default::default()
        };
        self.results = api.search_construction_depots(&query).unwrap_or_default();
        self.selected_idx = 0;
        self.scroll_offset = 0;
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_input.trim().is_empty() && self.pinned_ids.is_empty() {
            self.results.clear();
            return;
        }
        let pending = Rc::clone(&self.pending_search);
        let api = api.clone();
        let query = ConstructionQuery {
            name: if self.search_input.trim().is_empty() { None } else { Some(self.search_input.trim().to_string()) },
            limit: Some(50),
            ..Default::default()
        };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_construction_depots(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        // Search input mode
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
            KeyCode::Tab => return ViewEvent::NextTab,
            KeyCode::BackTab => return ViewEvent::PrevTab,
            KeyCode::Char('f') | KeyCode::Char('/') => {
                self.search_active = true;
                self.search_input.clear();
                self.focus = FocusArea::List;
                return ViewEvent::Consumed;
            }
            KeyCode::Enter => {
                if matches!(key.modifiers, KeyModifiers::NONE) {
                    if matches!(self.focus, FocusArea::List) {
                        self.focus = FocusArea::Detail;
                        self.detail_scroll = 0;
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
                    }
                }
                KeyCode::Char('s') | KeyCode::Down => {
                    if self.selected_idx + 1 < self.display_count() {
                        self.selected_idx += 1;
                    }
                }
                KeyCode::Char('p') => {
                    if self.display_count() > 0 {
                        self.toggle_pin(api);
                    }
                }
                KeyCode::Char('a') => {
                    self.pin_local_depot(api, journal);
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
        if row < self.scroll_offset {
            self.scroll_offset = row;
        } else if row >= self.scroll_offset + visible_height {
            self.scroll_offset = row + 1 - visible_height;
        }
    }

    fn build_list_lines(&self) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Search bar
        let search_bar = if self.search_active {
            format!("Search: {}█", self.search_input)
        } else if self.search_input.is_empty() {
            "Search: (f or / to search)".into()
        } else {
            format!("Search: {}", self.search_input)
        };
        lines.push(Line::from(Span::styled(
            search_bar,
            Style::default().fg(if self.search_active {
                Color::Yellow
            } else {
                Color::DarkGray
            }),
        )));
        lines.push(Line::from(""));

        if self.pinned_results.is_empty() && self.results.is_empty() {
            lines.push(Line::from(Span::styled(
                "No results. Search or press 'a' when docked at a construction site.",
                Style::default().fg(Color::DarkGray),
            )));
            return lines;
        }

        let search_deduped: Vec<&ConstructionDepotResponse> = self
            .results
            .iter()
            .filter(|r| !self.pinned_ids.contains(&r.market_id))
            .collect();
        let has_sep = !self.pinned_results.is_empty() && !search_deduped.is_empty();

        let mut row: usize = 0;

        // Pinned entries
        for depot in &self.pinned_results {
            let is_selected = row == self.selected_idx;
            lines.push(depot_list_line(depot, true, is_selected, &self.pinned_ids));
            row += 1;
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

        // Search results
        for depot in &search_deduped {
            let is_selected = row == self.selected_idx;
            lines.push(depot_list_line(depot, false, is_selected, &self.pinned_ids));
            row += 1;
        }

        lines
    }

    fn build_detail_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines: Vec<Line<'static>> = Vec::new();

        // Check local journal for fresher data first
        let selected = self.selected_item();
        let local: Option<&crate::journal_reader::ConstructionDepotData> = selected
            .and_then(|d| journal.construction_depots.get(&d.market_id));

        if let Some(local) = local {
            // Build from local journal data (most up-to-date)
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
                "Press 'a' when docked at a construction depot",
                Style::default().fg(Color::DarkGray),
            )));
            lines.push(Line::from(Span::styled(
                "to track it and submit its data to the server.",
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect, journal: &JournalData) {
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

        let list_lines = self.build_list_lines();
        let list_paragraph = Paragraph::new(list_lines)
            .block(
                Block::default()
                    .title(" Construction Sites (f: search | a: track docked | p: pin | Enter: details) ")
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

fn depot_list_line(
    depot: &ConstructionDepotResponse,
    is_pinned: bool,
    is_selected: bool,
    _pinned_ids: &HashSet<i64>,
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

    let status_icon = if depot.construction_complete {
        "✓ "
    } else if depot.construction_failed {
        "✗ "
    } else {
        "  "
    };
    let status_color = if depot.construction_complete {
        Color::Green
    } else if depot.construction_failed {
        Color::Red
    } else {
        Color::DarkGray
    };

    let pin_str = if is_pinned { "★ " } else { "  " };
    let pct = format!("{:>5.1}%", depot.progress * 100.0);
    let system = depot.system_name.clone();
    let name = depot.station_name.clone();

    Line::from(vec![
        Span::styled(pin_str, accent_style),
        Span::styled(status_icon, Style::default().fg(status_color)),
        Span::styled(name, name_style),
        Span::styled(
            format!("  {}", system),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(format!("  {}", pct), progress_color_style(depot.progress)),
    ])
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
        let frac = if res.required_amount == 0 {
            1.0f32
        } else {
            res.provided_amount as f32 / res.required_amount as f32
        };
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
        lines.push(Line::from(vec![
            Span::styled("  Updated ", Style::default().fg(Color::Cyan)),
            Span::styled(
                depot.last_updated[..19].to_string(),
                Style::default().fg(Color::DarkGray),
            ),
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
        let frac = if res.required_amount == 0 {
            1.0f32
        } else {
            res.provided_amount as f32 / res.required_amount as f32
        };
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
    let status = if complete {
        " ✓ Complete"
    } else if failed {
        " ✗ Failed"
    } else {
        ""
    };
    lines.push(Line::from(Span::styled(
        format!("{}{}", station_name, status),
        Style::default()
            .fg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD),
    )));

    // Overall progress bar
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
    let color = if frac >= 1.0 {
        Color::Green
    } else if frac > 0.0 {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    // Mini progress bar (10 chars)
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
        Span::styled(format!("  {:<22}", display_name), Style::default().fg(Color::White)),
        Span::styled(
            format!("{:>6}/{:<6}", provided, required),
            Style::default().fg(color),
        ),
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
        Style::default()
            .fg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD),
    ))
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
