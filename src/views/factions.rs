use crate::event_shim::{KeyCode, KeyEvent};
use edcas_common::api::{FactionQuery, FactionResponse, InfluencePoint};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};
use std::cmp::Ordering;
use std::collections::HashSet;

use crate::api_client::ApiClient;
use crate::views::util::{fmt_ts_short as fmt_ts, truncate, FocusArea, SearchState};
use crate::views::ViewEvent;

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq)]
enum DetailTab {
    Info,
    Systems,
    History,
}

#[derive(Clone, Copy, PartialEq)]
enum SystemSort {
    Name,
    Influence,
    ActiveState,
    Pending,
    Updated,
}

impl SystemSort {
    fn default_dir(self) -> SortDir {
        match self {
            Self::Influence | Self::Updated => SortDir::Desc,
            _ => SortDir::Asc,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum SortDir { Asc, Desc }

impl DetailTab {
    fn next(self) -> Option<Self> {
        match self {
            Self::Info    => Some(Self::Systems),
            Self::Systems => Some(Self::History),
            Self::History => None,
        }
    }
    fn prev(self) -> Option<Self> {
        match self {
            Self::Info    => None,
            Self::Systems => Some(Self::Info),
            Self::History => Some(Self::Systems),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Info    => "Info",
            Self::Systems => "Systems",
            Self::History => "History",
        }
    }
}

pub struct FactionsView {
    search_query: String,
    search_state: SearchState,
    results: Vec<FactionResponse>,
    pinned_names: HashSet<String>,
    pinned_results: Vec<FactionResponse>,
    selected_idx: usize,
    selected_system: usize,
    list_scroll: usize,
    detail_scroll: usize,
    status_msg: String,
    focus: FocusArea,
    detail_tab: DetailTab,
    loading: bool,
    spinner_frame: u8,
    system_sort: SystemSort,
    system_sort_dir: SortDir,
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<arboard::Clipboard>,
    #[cfg(not(target_arch = "wasm32"))]
    loading_pins: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_pins: Arc<Mutex<Option<Result<Vec<FactionResponse>, String>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pending_search: Arc<Mutex<Option<Result<Vec<FactionResponse>, String>>>>,
    #[cfg(target_arch = "wasm32")]
    pending_search: Rc<RefCell<Option<Vec<FactionResponse>>>>,

    // History tab state
    history_faction_name: String,
    history_system_name: String,
    /// Pre-computed chart points: (days_since_first, influence_percent)
    history_data: Vec<(f64, f64)>,
    history_time_bounds: [f64; 2],
    history_y_bounds: [f64; 2],
    history_x_labels: Vec<String>,
    history_y_labels: Vec<String>,
    #[cfg(not(target_arch = "wasm32"))]
    loading_history: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_history: Arc<Mutex<Option<Result<Vec<InfluencePoint>, String>>>>,
}

impl FactionsView {
    pub fn new() -> Self {
        let pins = crate::pins::Pins::load();
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            pinned_names: pins.factions,
            pinned_results: Vec::new(),
            selected_idx: 0,
            selected_system: 0,
            list_scroll: 0,
            detail_scroll: 0,
            status_msg: "/ or f: search  |  p: pin/unpin".into(),
            focus: FocusArea::List,
            detail_tab: DetailTab::Info,
            loading: false,
            spinner_frame: 0,
            system_sort: SystemSort::Influence,
            system_sort_dir: SortDir::Desc,
            // Kept alive for the duration of the view so the X11/Wayland
            // background clipboard-serving thread keeps running after set_text.
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: arboard::Clipboard::new().ok(),
            #[cfg(not(target_arch = "wasm32"))]
            loading_pins: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_pins: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            pending_search: Arc::new(Mutex::new(None)),
            #[cfg(target_arch = "wasm32")]
            pending_search: Rc::new(RefCell::new(None)),
            history_faction_name: String::new(),
            history_system_name: String::new(),
            history_data: Vec::new(),
            history_time_bounds: [0.0, 1.0],
            history_y_bounds: [0.0, 100.0],
            history_x_labels: Vec::new(),
            history_y_labels: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            loading_history: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_history: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_search(&mut self) {
        if let Some(result) = self.pending_pins.lock().unwrap().take() {
            self.loading_pins = false;
            match result {
                Ok(results) => {
                    self.pinned_results = results;
                    if self.status_msg.starts_with("Loading") {
                        self.status_msg = "/ or f: search  |  p: pin/unpin".into();
                    }
                }
                Err(e) => { self.status_msg = format!("API error loading pins: {e}"); }
            }
        }
        if let Some(result) = self.pending_search.lock().unwrap().take() {
            self.loading = false;
            match result {
                Ok(results) => {
                    let count = results.len();
                    self.results = results;
                    self.selected_idx = 0;
                    self.selected_system = 0;
                    self.list_scroll = 0;
                    self.detail_scroll = 0;
                    self.focus = FocusArea::List;
                    self.detail_tab = DetailTab::Info;
                    self.status_msg = if count == 0 {
                        format!("No factions found for '{}'", self.search_query)
                    } else {
                        format!("{count} faction(s) found")
                    };
                }
                Err(e) => { self.status_msg = format!("API error: {e}"); }
            }
        } else if self.loading {
            self.spinner_frame = self.spinner_frame.wrapping_add(1);
        }

        let history_result = self.pending_history.lock().unwrap().take();
        if let Some(result) = history_result {
            self.loading_history = false;
            match result {
                Ok(points) => self.set_history_data(points),
                Err(e) => {
                    self.history_data.clear();
                    self.history_x_labels = vec![format!("Error: {e}")];
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending_search.borrow_mut().take() {
            self.loading = false;
            let count = results.len();
            self.results = results;
            self.selected_idx = 0;
            self.selected_system = 0;
            self.list_scroll = 0;
            self.detail_scroll = 0;
            self.focus = FocusArea::List;
            self.detail_tab = DetailTab::Info;
            self.status_msg = if count == 0 {
                format!("No factions found for '{}'", self.search_query)
            } else {
                format!("{count} faction(s) found")
            };
        } else if self.loading {
            self.spinner_frame = self.spinner_frame.wrapping_add(1);
        }
    }

    pub fn on_enter(&mut self, api: &ApiClient) {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.pinned_names.is_empty() && self.pinned_results.is_empty() && !self.loading_pins {
            self.refresh_pins(api);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_pins(&mut self, api: &ApiClient) {
        self.loading_pins = true;
        self.pinned_results.clear();
        self.status_msg = "Loading pinned factions…".into();
        let pending = Arc::clone(&self.pending_pins);
        let api_owned = api.clone();
        let names: Vec<String> = self.pinned_names.iter().cloned().collect();
        api.spawn(async move {
            let mut results = Vec::new();
            for name in names {
                let query = FactionQuery { name: Some(name.clone()), limit: Some(10) };
                if let Ok(factions) = api_owned.search_factions(&query).await {
                    if let Some(f) = factions.into_iter().find(|f| f.name == name) {
                        results.push(f);
                    }
                }
            }
            results.sort_by(|a, b| a.name.cmp(&b.name));
            *pending.lock().unwrap() = Some(Ok(results));
        });
    }

    fn save_pins(&self) {
        let mut pins = crate::pins::Pins::load();
        pins.factions = self.pinned_names.clone();
        pins.save();
    }

    fn display_count(&self) -> usize {
        let n_search = self.results.iter()
            .filter(|f| !self.pinned_names.contains(&f.name))
            .count();
        self.pinned_results.len() + n_search
    }

    fn selected_faction(&self) -> Option<&FactionResponse> {
        let n = self.pinned_results.len();
        if self.selected_idx < n {
            self.pinned_results.get(self.selected_idx)
        } else {
            let j = self.selected_idx - n;
            self.results.iter()
                .filter(|f| !self.pinned_names.contains(&f.name))
                .nth(j)
        }
    }

    fn toggle_pin(&mut self, api: &ApiClient) {
        let n = self.pinned_results.len();
        if self.selected_idx < n {
            // Unpin
            let name = self.pinned_results[self.selected_idx].name.clone();
            self.pinned_names.remove(&name);
            self.pinned_results.remove(self.selected_idx);
            let total = self.display_count();
            if total > 0 {
                self.selected_idx = self.selected_idx.min(total - 1);
            } else {
                self.selected_idx = 0;
            }
        } else {
            // Pin
            let j = self.selected_idx - n;
            if let Some(faction) = self.results.iter()
                .filter(|f| !self.pinned_names.contains(&f.name))
                .nth(j)
                .cloned()
            {
                let name = faction.name.clone();
                self.pinned_names.insert(name.clone());
                self.pinned_results.push(faction);
                self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
                if let Some(pos) = self.pinned_results.iter().position(|f| f.name == name) {
                    self.selected_idx = pos;
                }
            } else if !self.pinned_names.is_empty() {
                #[cfg(not(target_arch = "wasm32"))]
                self.refresh_pins(api);
            }
        }
        self.save_pins();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_query.is_empty() {
            self.status_msg = "Enter a search term first".into();
            return;
        }
        self.loading = true;
        self.spinner_frame = 0;
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let pending = Arc::clone(&self.pending_search);
        let api_owned = api.clone();
        let query = FactionQuery { name: Some(self.search_query.clone()), limit: Some(100) };
        api.spawn(async move {
            let result = api_owned.search_factions(&query).await.map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn do_search(&mut self, api: &ApiClient) {
        if self.search_query.is_empty() {
            self.status_msg = "Enter a search term first".into();
            return;
        }
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let pending = Rc::clone(&self.pending_search);
        let api = api.clone();
        let query = FactionQuery { name: Some(self.search_query.clone()), limit: Some(100) };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_factions(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_history(&mut self, api: &ApiClient) {
        // Collect needed data while self is borrowed immutably, then release the borrow.
        let info = {
            let faction = match self.selected_faction() {
                Some(f) => f,
                None => return,
            };
            let indices = self.sorted_system_indices(faction);
            let sys_i = match indices.get(self.selected_system).copied() {
                Some(i) => i,
                None => return,
            };
            match faction.presences.get(sys_i) {
                Some(p) => (faction.name.clone(), p.system_name.clone(), p.system_address),
                None => return,
            }
        };
        let (faction_name, system_name, system_address) = info;

        self.history_faction_name = faction_name.clone();
        self.history_system_name = system_name;
        self.loading_history = true;
        self.history_data.clear();
        self.history_x_labels.clear();

        let pending = Arc::clone(&self.pending_history);
        let api_clone = api.clone();

        api.spawn(async move {
            let result = api_clone
                .fetch_faction_influence_history(&faction_name, system_address, 90)
                .await
                .map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    fn set_history_data(&mut self, mut points: Vec<InfluencePoint>) {
        if points.is_empty() {
            self.history_data.clear();
            self.history_x_labels = vec!["No data yet".to_string()];
            return;
        }

        // Deduplicate: keep one point per hour (last one wins) to reduce noise.
        points.dedup_by(|a, b| {
            let ta = a.timestamp.timestamp() / 3600;
            let tb = b.timestamp.timestamp() / 3600;
            ta == tb
        });

        let min_ts = points[0].timestamp.timestamp() as f64;
        let max_ts = points.last().unwrap().timestamp.timestamp() as f64;
        let total_days = ((max_ts - min_ts) / 86400.0).max(0.5);

        self.history_data = points
            .iter()
            .map(|p| {
                let x = (p.timestamp.timestamp() as f64 - min_ts) / 86400.0;
                let y = (p.influence * 100.0) as f64;
                (x, y)
            })
            .collect();

        self.history_time_bounds = [0.0, total_days];

        let values: Vec<f64> = self.history_data.iter().map(|(_, y)| *y).collect();
        let min_y = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_y = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let padding = ((max_y - min_y) * 0.15).max(2.0);
        self.history_y_bounds = [(min_y - padding).max(0.0), (max_y + padding).min(100.0)];

        let n_x = 5usize;
        let start_ts = chrono::DateTime::from_timestamp(points[0].timestamp.timestamp(), 0)
            .unwrap_or(points[0].timestamp);
        self.history_x_labels = (0..n_x)
            .map(|i| {
                let secs = (total_days * i as f64 / (n_x - 1) as f64 * 86400.0) as i64;
                let dt = start_ts + chrono::Duration::seconds(secs);
                dt.format("%m/%d").to_string()
            })
            .collect();

        let y_lo = self.history_y_bounds[0];
        let y_hi = self.history_y_bounds[1];
        self.history_y_labels = (0..5)
            .map(|i| format!("{:.0}%", y_lo + (y_hi - y_lo) * i as f64 / 4.0))
            .collect();
    }

    fn sorted_system_indices(&self, faction: &FactionResponse) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..faction.presences.len()).collect();
        indices.sort_by(|&a, &b| {
            let pa = &faction.presences[a];
            let pb = &faction.presences[b];
            let ord = match self.system_sort {
                SystemSort::Name => pa.system_name.cmp(&pb.system_name),
                SystemSort::Influence => pa.influence
                    .partial_cmp(&pb.influence)
                    .unwrap_or(Ordering::Equal),
                SystemSort::ActiveState => pa.active_states.first().map(String::as_str).unwrap_or("")
                    .cmp(pb.active_states.first().map(String::as_str).unwrap_or("")),
                SystemSort::Pending => pa.pending_states.first().map(String::as_str).unwrap_or("")
                    .cmp(pb.pending_states.first().map(String::as_str).unwrap_or("")),
                SystemSort::Updated => pa.updated_at.cmp(&pb.updated_at),
            };
            if self.system_sort_dir == SortDir::Desc { ord.reverse() } else { ord }
        });
        indices
    }

    fn visual_row_of_selected(&self) -> usize {
        let header = 3usize;
        let n = self.pinned_results.len();
        let has_separator = n > 0 && !self.results.is_empty();
        if self.selected_idx < n {
            header + self.selected_idx
        } else {
            header + self.selected_idx + if has_separator { 1 } else { 0 }
        }
    }

    fn build_list_lines(&self) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                self.search_query.clone(),
                Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                match self.search_state { SearchState::Typing => "_", SearchState::Idle => "" },
                Style::default().fg(Color::Yellow),
            ),
        ]));
        if self.loading {
            const FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
            let ch = FRAMES[(self.spinner_frame as usize) % FRAMES.len()];
            lines.push(Line::from(vec![
                Span::styled(format!("{ch} "), Style::default().fg(Color::Rgb(255, 140, 0))),
                Span::styled(self.status_msg.clone(), Style::default().fg(Color::DarkGray)),
            ]));
        } else {
            lines.push(Line::from(Span::styled(self.status_msg.clone(), Style::default().fg(Color::DarkGray))));
        }
        lines.push(Line::from(""));

        let n = self.pinned_results.len();

        // Pinned section
        for (i, faction) in self.pinned_results.iter().enumerate() {
            let selected = i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };
            let sys_label = match faction.presences.len() { 1 => "1 system".to_string(), s => format!("{s} systems") };
            lines.push(Line::from(Span::styled(
                format!(" ★ {:<36} {}", faction.name, sys_label),
                style,
            )));
        }

        // Separator + search section
        let deduped: Vec<&FactionResponse> = self.results.iter()
            .filter(|f| !self.pinned_names.contains(&f.name))
            .collect();

        if !deduped.is_empty() && n > 0 {
            lines.push(Line::from(Span::styled(
                "─── Search ──────────────────────────────",
                Style::default().fg(Color::DarkGray),
            )));
        } else if deduped.is_empty() && n == 0 {
            lines.push(Line::from("No results."));
            lines.push(Line::from("Press Enter to search."));
            return lines;
        }

        for (j, faction) in deduped.iter().enumerate() {
            let i = n + j;
            let selected = i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let sys_label = match faction.presences.len() { 1 => "1 system".to_string(), s => format!("{s} systems") };
            lines.push(Line::from(Span::styled(
                format!("   {:<36} {}", faction.name, sys_label),
                style,
            )));
        }

        lines
    }

    fn build_detail_header_lines(&self) -> Vec<Line<'static>> {
        let tab_active = Style::default()
            .fg(Color::Black)
            .bg(Color::Rgb(255, 140, 0))
            .add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [DetailTab::Info, DetailTab::Systems, DetailTab::History];
        let tab_spans: Vec<Span> = tabs
            .iter()
            .flat_map(|&t| {
                let style = if t == self.detail_tab { tab_active } else { tab_inactive };
                [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
            })
            .collect();

        let mut lines = vec![Line::from(tab_spans)];

        if let Some(faction) = self.selected_faction() {
            lines.push(Line::from(Span::styled(
                format!("── {} ──", faction.name),
                Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
            )));
        }

        lines
    }

    fn build_detail_body_lines(&self) -> Vec<Line<'static>> {
        let Some(faction) = self.selected_faction() else {
            return vec![Line::from(Span::styled(
                "Select a faction from the list.",
                Style::default().fg(Color::DarkGray),
            ))];
        };

        match self.detail_tab {
            DetailTab::Info => info_body(faction),
            DetailTab::Systems => {
                let indices = self.sorted_system_indices(faction);
                systems_body(faction, self.selected_system, self.focus == FocusArea::Detail, &indices, self.system_sort, self.system_sort_dir)
            }
            DetailTab::History => vec![], // rendered directly as a Chart in render()
        }
    }

    /// Keep detail_scroll so the selected system row stays visible.
    /// Body layout: line 0 = column header, line 1 = separator, line N+2 = presence[N].
    fn sync_system_scroll(&mut self) {
        self.detail_scroll = self.selected_system.saturating_sub(1);
    }

    pub fn prefill_search(&mut self, name: &str, api: &ApiClient) {
        self.search_query = name.to_string();
        self.search_state = SearchState::Idle;
        self.do_search(api);
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient) -> ViewEvent {
        if matches!(self.search_state, SearchState::Typing) {
            match key.code {
                KeyCode::Esc => {
                    self.search_state = SearchState::Idle;
                    self.status_msg = "Search cancelled".into();
                }
                KeyCode::Enter => {
                    self.search_state = SearchState::Idle;
                    self.do_search(api);
                }
                KeyCode::Backspace => { self.search_query.pop(); }
                KeyCode::Char(c)   => { self.search_query.push(c); }
                _ => {}
            }
            return ViewEvent::Consumed;
        }

        match key.code {
            KeyCode::Char('/') | KeyCode::Char('f') => {
                self.search_query.clear();
                self.search_state = SearchState::Typing;
                self.status_msg = "Typing… (Enter to search, Esc to cancel)".into();
            }
            KeyCode::Char('p') => {
                if self.display_count() > 0 {
                    self.toggle_pin(api);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => match self.focus {
                FocusArea::List => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        self.selected_system = 0;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Info;
                    }
                }
                FocusArea::Detail if self.detail_tab == DetailTab::Systems => {
                    self.selected_system = self.selected_system.saturating_sub(1);
                    self.sync_system_scroll();
                }
                FocusArea::Detail => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.focus {
                FocusArea::List => {
                    if self.selected_idx + 1 < self.display_count() {
                        self.selected_idx += 1;
                        self.selected_system = 0;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Info;
                    }
                }
                FocusArea::Detail if self.detail_tab == DetailTab::Systems => {
                    let max = self.selected_faction()
                        .map(|f| f.presences.len().saturating_sub(1))
                        .unwrap_or(0);
                    self.selected_system = (self.selected_system + 1).min(max);
                    self.sync_system_scroll();
                }
                FocusArea::Detail => {
                    self.detail_scroll += 1;
                }
            },
            KeyCode::PageUp => match self.focus {
                FocusArea::List => { self.selected_idx = self.selected_idx.saturating_sub(10); }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(10); }
            },
            KeyCode::PageDown => match self.focus {
                FocusArea::List => {
                    let max = self.display_count().saturating_sub(1);
                    self.selected_idx = (self.selected_idx + 10).min(max);
                }
                FocusArea::Detail => { self.detail_scroll += 10; }
            },
            KeyCode::Char('c') => {
                if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Systems {
                    if let Some(faction) = self.selected_faction() {
                        if let Some(presence) = faction.presences.get(self.selected_system) {
                            let name = presence.system_name.trim().to_string();
                            #[cfg(not(target_arch = "wasm32"))]
                            if let Some(cb) = self.clipboard.as_mut() {
                                let _ = cb.set_text(&name);
                            }
                        }
                    }
                    return ViewEvent::Consumed;
                }
            }
            KeyCode::Tab => {
                match self.focus {
                    FocusArea::List => {
                        if self.display_count() > 0 {
                            self.focus = FocusArea::Detail;
                            self.detail_scroll = 0;
                        }
                    }
                    FocusArea::Detail => { self.focus = FocusArea::List; }
                }
            }
            KeyCode::Char('d') | KeyCode::Right => {
                if self.focus == FocusArea::Detail {
                    if let Some(next) = self.detail_tab.next() {
                        self.detail_tab = next;
                        self.detail_scroll = 0;
                        #[cfg(not(target_arch = "wasm32"))]
                        if next == DetailTab::History {
                            self.fetch_history(api);
                        }
                    }
                }
            }
            KeyCode::Char('a') | KeyCode::Left => {
                if self.focus == FocusArea::Detail {
                    if let Some(prev) = self.detail_tab.prev() {
                        self.detail_tab = prev;
                        self.detail_scroll = 0;
                    }
                }
            }
            KeyCode::Char(c @ '1'..='5')
                if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Systems =>
            {
                let col = match c {
                    '1' => SystemSort::Name,
                    '2' => SystemSort::Influence,
                    '3' => SystemSort::ActiveState,
                    '4' => SystemSort::Pending,
                    '5' => SystemSort::Updated,
                    _ => unreachable!(),
                };
                if col == self.system_sort {
                    self.system_sort_dir = match self.system_sort_dir {
                        SortDir::Asc => SortDir::Desc,
                        SortDir::Desc => SortDir::Asc,
                    };
                } else {
                    self.system_sort = col;
                    self.system_sort_dir = col.default_dir();
                }
                self.selected_system = 0;
                self.detail_scroll = 0;
            }
            _ => {}
        }

        ViewEvent::None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        // ── Left: faction list ───────────────────────────────────
        let list_lines = self.build_list_lines();
        let list_height = chunks[0].height.saturating_sub(2) as usize;
        let row = self.visual_row_of_selected();
        let list_scroll = if row + 1 >= self.list_scroll + list_height {
            (row + 2).saturating_sub(list_height)
        } else if row < self.list_scroll {
            row.saturating_sub(1)
        } else {
            self.list_scroll
        };

        frame.render_widget(
            Paragraph::new(list_lines)
                .block(
                    Block::default()
                        .title(" Factions ")
                        .borders(Borders::ALL)
                        .border_style(if self.focus == FocusArea::List {
                            active_border
                        } else {
                            inactive_border
                        }),
                )
                .scroll((list_scroll as u16, 0)),
            chunks[0],
        );

        // ── Right: detail ────────────────────────────────────────
        let detail_block = Block::default()
            .title(" Faction Details — Enter to search ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail {
                active_border
            } else {
                inactive_border
            });
        let detail_inner = detail_block.inner(chunks[1]);
        frame.render_widget(detail_block, chunks[1]);

        let header_lines = self.build_detail_header_lines();
        let header_height = header_lines.len() as u16;

        let detail_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .split(detail_inner);

        frame.render_widget(Paragraph::new(header_lines), detail_split[0]);

        if self.detail_tab == DetailTab::History {
            self.render_history_chart(frame, detail_split[1]);
        } else {
            let body_lines = self.build_detail_body_lines();
            let body_height = detail_split[1].height as usize;
            let body_max_scroll = body_lines.len().saturating_sub(body_height);
            frame.render_widget(
                Paragraph::new(body_lines)
                    .scroll((self.detail_scroll.min(body_max_scroll) as u16, 0)),
                detail_split[1],
            );
        }
    }

    fn render_history_chart(&self, frame: &mut Frame, area: Rect) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.loading_history {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "Loading influence history…",
                    Style::default().fg(Color::DarkGray),
                ))),
                area,
            );
            return;
        }

        if self.history_data.len() < 2 {
            let msg = if self.history_faction_name.is_empty() {
                "Navigate to Systems tab, select a system, then press d to load history."
            } else if self.history_data.is_empty() {
                "No history data recorded yet for this faction-system pair."
            } else {
                "Not enough data points to draw a chart yet."
            };
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    msg,
                    Style::default().fg(Color::DarkGray),
                ))),
                area,
            );
            return;
        }

        let x_labels: Vec<Span> = self
            .history_x_labels
            .iter()
            .map(|s| Span::raw(s.as_str()))
            .collect();
        let y_labels: Vec<Span> = self
            .history_y_labels
            .iter()
            .map(|s| Span::raw(s.as_str()))
            .collect();

        let dataset = Dataset::default()
            .name(format!(
                "{} in {}",
                self.history_faction_name, self.history_system_name
            ))
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Rgb(255, 140, 0)))
            .data(&self.history_data);

        let chart = Chart::new(vec![dataset])
            .x_axis(
                Axis::default()
                    .title("Date")
                    .bounds(self.history_time_bounds)
                    .labels(x_labels)
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .y_axis(
                Axis::default()
                    .title("Influence %")
                    .bounds(self.history_y_bounds)
                    .labels(y_labels)
                    .style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(chart, area);
    }
}

// ── Info tab ─────────────────────────────────────────────────────────────────

fn info_body(faction: &FactionResponse) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    if let Some(ref gov) = faction.government {
        lines.push(section_header(&format!("Government: {gov}")));
        for l in government_lines(gov) {
            lines.push(l);
        }
        lines.push(Line::from(""));
    }

    if let Some(ref alleg) = faction.allegiance {
        lines.push(section_header(&format!("Allegiance: {alleg}")));
        for l in allegiance_lines(alleg) {
            lines.push(l);
        }
        lines.push(Line::from(""));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No government / allegiance data available.",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

fn section_header(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        text.to_owned(),
        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
    ))
}

fn bullet(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  · {text}"),
        Style::default().fg(Color::White),
    ))
}

fn dim(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(Color::DarkGray),
    ))
}

fn label_line(label: &str, value: &str, color: Color) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {:<14}", label), Style::default().fg(Color::DarkGray)),
        Span::styled(value.to_owned(), Style::default().fg(color)),
    ])
}

fn government_lines(gov: &str) -> Vec<Line<'static>> {
    match gov {
        "Anarchy" => vec![
            dim("No formal laws or enforcement. Individual strength determines survival."),
            dim("Security forces are absent or entirely corrupt."),
            Line::from(""),
            label_line("Illegal:", "Nothing — no laws are enforced", Color::Green),
            label_line("Legal:", "Everything, including narcotics, weapons, slaves", Color::Cyan),
            label_line("Security:", "None — pirates thrive here", Color::Red),
            label_line("Trade risk:", "Very high — no protection for traders", Color::Red),
        ],
        "Communism" => vec![
            dim("State ownership of all production. Resources are allocated centrally"),
            dim("for the common good. Equality is prioritised over individual freedom."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, combat stabilisers, civilian weapons", Color::Yellow),
            label_line("Legal:", "Most standard goods and commodities", Color::Cyan),
            label_line("Security:", "Moderate to high", Color::Green),
            label_line("Trade:", "State-controlled; black markets common", Color::DarkGray),
        ],
        "Confederacy" => vec![
            dim("A loose alliance of autonomous member states sharing common defence"),
            dim("and trade agreements. Local authorities retain significant autonomy."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, some weapons (varies by station)", Color::Yellow),
            label_line("Legal:", "Most goods; weapon laws vary per port", Color::Cyan),
            label_line("Security:", "Moderate", Color::Green),
        ],
        "Corporate" => vec![
            dim("A megacorporation exercises governmental authority. Every policy"),
            dim("decision is driven by profit. Workers are assets; shareholders rule."),
            Line::from(""),
            label_line("Illegal:", "Narcotics (reduce productivity), unsanctioned weapons", Color::Yellow),
            label_line("Legal:", "Most goods if profitable; grey-market tolerant", Color::Cyan),
            label_line("Security:", "Moderate — corporate security forces", Color::Green),
            label_line("Trade:", "Strong markets; corporations control supply", Color::Green),
        ],
        "Cooperative" => vec![
            dim("Collectively owned and democratically managed. Members share profits"),
            dim("and governance equally. Community wellbeing is the primary goal."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, unregulated weapons", Color::Yellow),
            label_line("Legal:", "Most standard goods and services", Color::Cyan),
            label_line("Security:", "Moderate — community watch", Color::Green),
        ],
        "Democracy" => vec![
            dim("Elected representatives govern in the name of the people. Laws"),
            dim("balance individual freedoms with collective security."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, unregulated weapons, combat stabilisers", Color::Yellow),
            label_line("Legal:", "Most commercial goods; personal sidearms (licensed)", Color::Cyan),
            label_line("Security:", "Moderate to high", Color::Green),
        ],
        "Dictatorship" => vec![
            dim("A single autocrat holds absolute power. Order is maintained through"),
            dim("strict law enforcement and loyal security forces. Dissent is crushed."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, weapons, opposition materials, many goods", Color::Red),
            label_line("Legal:", "State-approved goods only", Color::Cyan),
            label_line("Security:", "Very high — harsh penalties", Color::Red),
            label_line("Trade:", "Restricted; state has monopoly on key goods", Color::Yellow),
        ],
        "Feudal" => vec![
            dim("Noble hierarchy governs by hereditary right. Vassals owe loyalty and"),
            dim("service to their liege lords. Might and lineage determine status."),
            Line::from(""),
            label_line("Illegal:", "Weapons for commoners, narcotics", Color::Yellow),
            label_line("Legal:", "Slaves (in many feudal systems), agricultural goods", Color::Cyan),
            label_line("Security:", "High for nobility; low for common areas", Color::Yellow),
        ],
        "Imperial" => vec![
            dim("Imperial hierarchy based on rank, honour and duty. Service to the"),
            dim("Empire is paramount. Social mobility exists through demonstrated loyalty."),
            Line::from(""),
            label_line("Illegal:", "Chattel slaves, narcotics, advanced personal weapons", Color::Red),
            label_line("Legal:", "Imperial Slaves (bond-servants with rights and a contract)", Color::Cyan),
            label_line("Security:", "High — Imperial Navy patrols", Color::Green),
            label_line("Note:", "Imperial Slaves ≠ chattel slavery; bond ends after term", Color::DarkGray),
        ],
        "Patronage" => vec![
            dim("Powerful patrons extend protection in exchange for loyalty and service."),
            dim("Political power flows through personal relationships, not institutions."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, weapons (varies by patron)", Color::Yellow),
            label_line("Legal:", "Goods approved by the ruling patron", Color::Cyan),
            label_line("Security:", "Moderate — patron's private forces", Color::Yellow),
        ],
        "Prison Colony" => vec![
            dim("A penal authority administers this system. Security forces are wardens."),
            dim("Residents are either prisoners, guards, or support staff."),
            Line::from(""),
            label_line("Illegal:", "Almost everything beyond basic necessities", Color::Red),
            label_line("Legal:", "Heavily regulated basic goods only", Color::Cyan),
            label_line("Security:", "Maximum — entire system is a prison", Color::Red),
            label_line("Trade:", "Extremely restricted; smuggling very risky", Color::Red),
        ],
        "Theocracy" => vec![
            dim("Sacred law governs all aspects of life. Religious authorities hold"),
            dim("ultimate temporal power. Deviation from doctrine is a criminal offence."),
            Line::from(""),
            label_line("Illegal:", "Narcotics, personal weapons, goods deemed heretical", Color::Red),
            label_line("Legal:", "Goods and services aligned with religious doctrine", Color::Cyan),
            label_line("Security:", "Very high — zealous enforcement", Color::Red),
            label_line("Trade:", "Restricted to approved goods", Color::Yellow),
        ],
        "None" => vec![
            dim("No established government. Laws and security vary unpredictably."),
        ],
        other => vec![
            dim(&format!("Government type '{other}' — no detailed information available.")),
        ],
    }
}

fn allegiance_lines(alleg: &str) -> Vec<Line<'static>> {
    match alleg {
        "Federation" => vec![
            dim("Member of the Galactic Federation. Values democracy, individual rights"),
            dim("and rule of law. Bureaucratic governance; strong Federal Navy."),
            bullet("Federal credits and rank can be earned through missions"),
            bullet("Anti-slavery: chattel and Imperial slaves are illegal in Fed space"),
            bullet("Strong presence in the Sol neighbourhood and core worlds"),
        ],
        "Empire" => vec![
            dim("Subject of the Galactic Empire. Values honour, rank, duty and tradition."),
            dim("Social hierarchy is rigid but meritocratic through demonstrated service."),
            bullet("Imperial rank earned through trade, combat and patron missions"),
            bullet("Imperial Slaves are legal — a regulated bond-servant system"),
            bullet("Narcotics and chattel slaves are illegal in Imperial space"),
        ],
        "Alliance" => vec![
            dim("Member of the Alliance of Independent Systems. Values sovereignty,"),
            dim("cooperation and mutual defence without imposing cultural uniformity."),
            bullet("Loosely governed — member systems retain local laws"),
            bullet("Focused on exploration, trade and defence"),
            bullet("No unified stance on slaves or narcotics; local laws apply"),
        ],
        "Independent" => vec![
            dim("No allegiance to a major power. Laws and policies are set entirely"),
            dim("by the local governing faction. Expect wide variation between systems."),
            bullet("Check local station laws before trading sensitive goods"),
            bullet("Often targets of powerplay expansion from the major factions"),
        ],
        "Pilots Federation" | "PilotsFederation" => vec![
            dim("Governed by the Pilots Federation — the guild of licensed commanders."),
            dim("Mostly neutral; these systems serve as training grounds and safe zones."),
            bullet("Rebuy penalties do not apply in Pilots Federation space"),
            bullet("Combat is illegal — these are protected starter systems"),
        ],
        other => vec![
            dim(&format!("Allegiance '{other}' — no detailed information available.")),
        ],
    }
}

// ── Systems tab ───────────────────────────────────────────────────────────────

fn systems_body(faction: &FactionResponse, selected: usize, focused: bool, sorted_indices: &[usize], sort_col: SystemSort, sort_dir: SortDir) -> Vec<Line<'static>> {
    if faction.presences.is_empty() {
        return vec![Line::from(Span::styled(
            "No system presence data available.",
            Style::default().fg(Color::DarkGray),
        ))];
    }

    let mut lines = Vec::new();

    let dir_char = if sort_dir == SortDir::Asc { "▲" } else { "▼" };
    let col_label = |col: SystemSort, name: &'static str| -> String {
        if col == sort_col { format!("{name}{dir_char}") } else { name.to_string() }
    };

    let hint = if focused {
        " — w/s: select  c: copy  1-5: sort"
    } else {
        ""
    };
    lines.push(Line::from(Span::styled(
        format!("{:<30} {:>7}  {:<20}  {:<20}  {:<11}{hint}",
            col_label(SystemSort::Name, "System"),
            col_label(SystemSort::Influence, "Inf%"),
            col_label(SystemSort::ActiveState, "Active State"),
            col_label(SystemSort::Pending, "Pending"),
            col_label(SystemSort::Updated, "Updated"),
        ),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(Span::styled(
        "─".repeat(78),
        Style::default().fg(Color::DarkGray),
    )));

    for (visual_i, &presence_i) in sorted_indices.iter().enumerate() {
        let p = &faction.presences[presence_i];
        let i = visual_i;
        let pct = p.influence * 100.0;
        let inf_color = if pct < 15.0 {
            Color::Red
        } else if pct < 40.0 {
            Color::Yellow
        } else {
            Color::Green
        };

        let active = if p.active_states.is_empty() {
            "—".to_string()
        } else {
            p.active_states.iter()
                .map(|s| super::annotate_faction_state(s, false))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let pending = if p.pending_states.is_empty() {
            "—".to_string()
        } else {
            p.pending_states.iter()
                .map(|s| super::annotate_faction_state(s, true))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let is_selected = focused && i == selected;

        let ts = fmt_ts(p.updated_at.as_ref());
        if is_selected {
            let row = format!(
                " {:<29} {:>5.1}%  {:<20}  {:<20}  {:<11}",
                truncate(&p.system_name, 29),
                pct,
                truncate(&active, 20),
                truncate(&pending, 20),
                ts,
            );
            lines.push(Line::from(Span::styled(
                row,
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(vec![
                Span::raw(format!(" {:<29} ", truncate(&p.system_name, 29))),
                Span::styled(format!("{:>5.1}%", pct), Style::default().fg(inf_color)),
                Span::raw(format!("  {:<20}  {:<20}  {}", truncate(&active, 20), pending, ts)),
            ]));
        }

        // Conflict / war info line
        if let Some(ref c) = p.conflict {
            let score_color = if c.our_won_days > c.opponent_won_days {
                Color::Green
            } else if c.our_won_days < c.opponent_won_days {
                Color::Red
            } else {
                Color::Yellow
            };
            let war_label = match c.war_type.as_str() {
                "Election" => "Election",
                "CivilWar" => "Civil War",
                _ => "War",
            };
            let score = format!("{}:{}", c.our_won_days, c.opponent_won_days);
            let winner = if c.our_won_days > c.opponent_won_days {
                "Winning"
            } else if c.our_won_days < c.opponent_won_days {
                "Losing"
            } else {
                "Tied"
            };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("   ↳ {war_label} vs "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    truncate(&c.opponent_name, 28),
                    Style::default().fg(Color::White),
                ),
                Span::styled("  Score: ", Style::default().fg(Color::DarkGray)),
                Span::styled(score, Style::default().fg(score_color).add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("  ({winner})"),
                    Style::default().fg(score_color),
                ),
            ]));
        }
    }

    lines
}

