use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{JournalData, StationData};
use edcas_common::api::{StationQuery, StationResponse};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use crate::api_client::ApiClient;
use crate::todo::TodoList;
use crate::views::util::{commodity_header_line, commodity_row, fmt_ts, normalize_commodity_name, raw_diff, truncate, FocusArea, MarketSortCol, SearchState, StationDetailTab as DetailTab};
use crate::views::ViewEvent;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};

enum ListItem<'a> {
    Api(&'a StationResponse),
    Journal(&'a StationData),
}

pub struct StationsView {
    search_query: String,
    search_state: SearchState,
    results: Vec<StationResponse>,
    pinned_ids: HashSet<i64>,
    pinned_results: Vec<StationResponse>,
    /// Stations fetched automatically on dock; used instead of the journal snapshot.
    auto_fetched: HashMap<i64, StationResponse>,
    /// Market IDs for which an auto-fetch was attempted (regardless of outcome),
    /// so we don't retry on every journal update.
    auto_fetch_attempted: HashSet<i64>,
    selected_idx: usize,
    list_scroll: usize,
    detail_scroll: usize,
    status_msg: String,
    focus: FocusArea,
    detail_tab: DetailTab,
    market_sort_col: MarketSortCol,
    market_sort_asc: bool,
    #[cfg(not(target_arch = "wasm32"))]
    clipboard: Option<arboard::Clipboard>,
    #[cfg(target_arch = "wasm32")]
    clipboard: (),
    #[cfg(not(target_arch = "wasm32"))]
    loading_pins: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_pins: Arc<Mutex<Option<Result<Vec<StationResponse>, String>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pending_search: Arc<Mutex<Option<Result<Vec<StationResponse>, String>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pending_auto_fetch: Arc<Mutex<Option<Result<Vec<StationResponse>, String>>>>,
    #[cfg(not(target_arch = "wasm32"))]
    auto_fetch_in_flight: Option<i64>,
    #[cfg(target_arch = "wasm32")]
    pending_search: Rc<RefCell<Option<Vec<StationResponse>>>>,
}

impl StationsView {
    pub fn new() -> Self {
        let pins = crate::pins::Pins::load();
        Self {
            search_query: String::new(),
            search_state: SearchState::Idle,
            results: Vec::new(),
            pinned_ids: pins.stations,
            pinned_results: Vec::new(),
            auto_fetched: HashMap::new(),
            auto_fetch_attempted: HashSet::new(),
            selected_idx: 0,
            list_scroll: 0,
            detail_scroll: 0,
            status_msg: "Press Enter to search  |  p: pin/unpin  |  c: copy system".into(),
            focus: FocusArea::List,
            detail_tab: DetailTab::Overview,
            market_sort_col: MarketSortCol::default(),
            market_sort_asc: true,
            #[cfg(not(target_arch = "wasm32"))]
            clipboard: arboard::Clipboard::new().ok(),
            #[cfg(target_arch = "wasm32")]
            clipboard: (),
            #[cfg(not(target_arch = "wasm32"))]
            loading_pins: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_pins: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            pending_search: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            pending_auto_fetch: Arc::new(Mutex::new(None)),
            #[cfg(not(target_arch = "wasm32"))]
            auto_fetch_in_flight: None,
            #[cfg(target_arch = "wasm32")]
            pending_search: Rc::new(RefCell::new(None)),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending_search.borrow_mut().take() {
            let count = results.len();
            self.results = results;
            self.selected_idx = 0;
            self.list_scroll = 0;
            self.detail_scroll = 0;
            self.focus = FocusArea::List;
            self.detail_tab = DetailTab::Overview;
            self.status_msg = if count == 0 {
                format!("No stations found for '{}'", self.search_query)
            } else {
                format!("{count} station(s) found")
            };
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
                        self.status_msg = "Press Enter to search  |  p: pin/unpin  |  c: copy system".into();
                    }
                }
                Err(e) => { self.status_msg = format!("API error loading pins: {e}"); }
            }
        }
        if let Some(result) = self.pending_search.lock().unwrap().take() {
            match result {
                Ok(results) => {
                    let count = results.len();
                    self.results = results;
                    self.selected_idx = 0;
                    self.list_scroll = 0;
                    self.detail_scroll = 0;
                    self.focus = FocusArea::List;
                    self.detail_tab = DetailTab::Overview;
                    self.status_msg = if count == 0 {
                        format!("No stations found for '{}'", self.search_query)
                    } else {
                        format!("{count} station(s) found")
                    };
                }
                Err(e) => { self.status_msg = format!("API error: {e}"); }
            }
        }
    }

    pub fn on_enter(&mut self, api: &ApiClient) {
        #[cfg(not(target_arch = "wasm32"))]
        if !self.pinned_ids.is_empty() && self.pinned_results.is_empty() && !self.loading_pins {
            self.refresh_pins(api);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_pins(&mut self, api: &ApiClient) {
        self.loading_pins = true;
        self.pinned_results.clear();
        self.status_msg = "Loading pinned stations…".into();
        let pending = Arc::clone(&self.pending_pins);
        let api_owned = api.clone();
        let ids: Vec<i64> = self.pinned_ids.iter().copied().collect();
        api.spawn(async move {
            let mut results = Vec::new();
            for mid in ids {
                let query = StationQuery { market_id: Some(mid), limit: Some(1), name: None, system_name: None };
                if let Ok(mut r) = api_owned.search_stations(&query).await {
                    if let Some(s) = r.pop() {
                        results.push(s);
                    }
                }
            }
            results.sort_by(|a, b| a.name.cmp(&b.name));
            *pending.lock().unwrap() = Some(Ok(results));
        });
    }

    fn save_pins(&self) {
        let mut pins = crate::pins::Pins::load();
        pins.stations = self.pinned_ids.clone();
        pins.save();
    }

    /// Triggered automatically when the player docks at a non-carrier station.
    /// Fetches full market data from the API and caches it so the journal snapshot
    /// is replaced with live data without requiring the user to pin the station.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn fetch_on_dock(&mut self, market_id: i64, api: &ApiClient) {
        if self.pinned_ids.contains(&market_id)
            || self.auto_fetch_attempted.contains(&market_id)
            || self.auto_fetch_in_flight == Some(market_id)
        {
            return;
        }
        self.auto_fetch_in_flight = Some(market_id);
        let pending = Arc::clone(&self.pending_auto_fetch);
        let api_owned = api.clone();
        api.spawn(async move {
            let query = StationQuery {
                market_id: Some(market_id),
                limit: Some(1),
                name: None,
                system_name: None,
            };
            let result = api_owned.search_stations(&query).await.map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_auto_fetch(&mut self) {
        if let Some(result) = self.pending_auto_fetch.lock().unwrap().take() {
            if let Some(id) = self.auto_fetch_in_flight.take() {
                self.auto_fetch_attempted.insert(id);
            }
            if let Ok(mut results) = result {
                if let Some(station) = results.pop() {
                    self.auto_fetched.insert(station.market_id, station);
                }
            }
        }
    }

    fn build_display_list<'a>(&'a self, history: &'a [StationData]) -> Vec<ListItem<'a>> {
        let search_ids: HashSet<i64> = self.results.iter()
            .filter(|s| !self.pinned_ids.contains(&s.market_id))
            .map(|s| s.market_id)
            .collect();
        let mut list: Vec<ListItem<'a>> = Vec::new();
        for s in &self.pinned_results {
            list.push(ListItem::Api(s));
        }
        for s in self.results.iter().filter(|s| !self.pinned_ids.contains(&s.market_id)) {
            list.push(ListItem::Api(s));
        }
        for s in history.iter().filter(|s| !self.pinned_ids.contains(&s.market_id) && !search_ids.contains(&s.market_id)) {
            if let Some(fetched) = self.auto_fetched.get(&s.market_id) {
                list.push(ListItem::Api(fetched));
            } else {
                list.push(ListItem::Journal(s));
            }
        }
        list
    }

    fn display_count(&self, history: &[StationData]) -> usize {
        self.build_display_list(history).len()
    }

    fn selected_item_with_history<'a>(&'a self, history: &'a [StationData]) -> Option<ListItem<'a>> {
        self.build_display_list(history).into_iter().nth(self.selected_idx)
    }

    fn toggle_pin(&mut self, api: &ApiClient, history: &[StationData]) {
        let n = self.pinned_results.len();
        if self.selected_idx < n {
            // Unpin
            let mid = self.pinned_results[self.selected_idx].market_id;
            self.pinned_ids.remove(&mid);
            self.pinned_results.remove(self.selected_idx);
            let total = self.display_count(history);
            if total > 0 {
                self.selected_idx = self.selected_idx.min(total - 1);
            } else {
                self.selected_idx = 0;
            }
        } else {
            let j = self.selected_idx - n;
            let deduped_search: Vec<&StationResponse> = self.results.iter()
                .filter(|s| !self.pinned_ids.contains(&s.market_id))
                .collect();
            if j < deduped_search.len() {
                // Pin from search
                let station = deduped_search[j].clone();
                let mid = station.market_id;
                self.pinned_ids.insert(mid);
                self.pinned_results.push(station);
                self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
                if let Some(pos) = self.pinned_results.iter().position(|s| s.market_id == mid) {
                    self.selected_idx = pos;
                }
            } else {
                // In history section — just mark as pinned; will load via API next refresh
                let search_ids: HashSet<i64> = self.results.iter().map(|s| s.market_id).collect();
                let hist_deduped: Vec<&StationData> = history.iter()
                    .filter(|s| !self.pinned_ids.contains(&s.market_id) && !search_ids.contains(&s.market_id))
                    .collect();
                let hi = j - deduped_search.len();
                if let Some(h) = hist_deduped.get(hi) {
                    let mid = h.market_id;
                    self.pinned_ids.insert(mid);
                    #[cfg(not(target_arch = "wasm32"))]
                    self.refresh_pins(api);
                    self.selected_idx = 0;
                } else if !self.pinned_ids.is_empty() {
                    #[cfg(not(target_arch = "wasm32"))]
                    self.refresh_pins(api);
                }
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
        self.status_msg = format!("Searching for '{}'…", self.search_query);
        let pending = Arc::clone(&self.pending_search);
        let api_owned = api.clone();
        let query = StationQuery {
            name: Some(self.search_query.clone()),
            system_name: None,
            market_id: None,
            limit: Some(50),
        };
        api.spawn(async move {
            let result = api_owned.search_stations(&query).await.map_err(|e| e.to_string());
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
        let query = StationQuery {
            name: Some(self.search_query.clone()),
            system_name: None,
            market_id: None,
            limit: Some(50),
        };
        wasm_bindgen_futures::spawn_local(async move {
            let results = api.search_stations(query).await;
            *pending.borrow_mut() = Some(results);
        });
    }

    fn build_list_lines(&self, history: &[StationData]) -> Vec<Line<'static>> {
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
        lines.push(Line::from(Span::styled(
            self.status_msg.clone(),
            Style::default().fg(Color::DarkGray),
        )));
        lines.push(Line::from(""));

        let n_pinned = self.pinned_results.len();
        let deduped_search: Vec<&StationResponse> = self.results.iter()
            .filter(|s| !self.pinned_ids.contains(&s.market_id))
            .collect();
        let search_ids: HashSet<i64> = self.results.iter().map(|s| s.market_id).collect();
        let hist_deduped: Vec<&StationData> = history.iter()
            .filter(|s| !self.pinned_ids.contains(&s.market_id) && !search_ids.contains(&s.market_id))
            .collect();

        let header = 3usize;

        // Pinned section
        for (i, station) in self.pinned_results.iter().enumerate() {
            let selected = self.focus == FocusArea::List && i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Yellow)
            };
            lines.push(Line::from(Span::styled(
                format!(" \u{2605} {:<26} {}", truncate(&station.name, 26), station.station_type.as_deref().unwrap_or("")),
                style,
            )));
        }

        // Search separator + results
        if !deduped_search.is_empty() && n_pinned > 0 {
            lines.push(Line::from(Span::styled(
                "\u{2500}\u{2500}\u{2500} Search \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                Style::default().fg(Color::DarkGray),
            )));
        }
        for (j, station) in deduped_search.iter().enumerate() {
            let i = n_pinned + j;
            let selected = self.focus == FocusArea::List && i == self.selected_idx;
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            lines.push(Line::from(Span::styled(
                format!("   {:<26} {}", truncate(&station.name, 26), station.station_type.as_deref().unwrap_or("")),
                style,
            )));
        }

        // History separator + entries
        let n_search = deduped_search.len();
        if !hist_deduped.is_empty() {
            let has_above = n_pinned > 0 || n_search > 0;
            if has_above {
                lines.push(Line::from(Span::styled(
                    "\u{2500}\u{2500}\u{2500} History \u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}",
                    Style::default().fg(Color::DarkGray),
                )));
            }
            for (k, station) in hist_deduped.iter().enumerate() {
                let i = n_pinned + n_search + k;
                let selected = self.focus == FocusArea::List && i == self.selected_idx;
                let has_live = self.auto_fetched.contains_key(&station.market_id);
                let style = if selected {
                    Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD)
                } else if has_live {
                    Style::default().fg(Color::Rgb(100, 220, 100))
                } else {
                    Style::default().fg(Color::Rgb(100, 180, 200))
                };
                let icon = if has_live { "\u{25CF}" } else { "\u{231a}" };
                lines.push(Line::from(Span::styled(
                    format!(" {} {:<26} {}", icon, truncate(&station.name, 26), &station.station_type),
                    style,
                )));
            }
        }

        if n_pinned == 0 && n_search == 0 && hist_deduped.is_empty() {
            lines.push(Line::from("No results."));
            lines.push(Line::from("Press Enter to search."));
        }

        let _ = header; // suppress unused warning
        lines
    }

    fn visual_row_of_selected(&self, history: &[StationData]) -> usize {
        let header = 3usize;
        let n_pinned = self.pinned_results.len();
        let deduped_search: Vec<&StationResponse> = self.results.iter()
            .filter(|s| !self.pinned_ids.contains(&s.market_id))
            .collect();
        let n_search = deduped_search.len();
        let search_ids: HashSet<i64> = self.results.iter().map(|s| s.market_id).collect();
        let n_hist = history.iter()
            .filter(|s| !self.pinned_ids.contains(&s.market_id) && !search_ids.contains(&s.market_id))
            .count();

        let has_search_sep = n_pinned > 0 && n_search > 0;
        let has_hist_sep = n_hist > 0 && (n_pinned > 0 || n_search > 0);

        let idx = self.selected_idx;
        if idx < n_pinned {
            header + idx
        } else if idx < n_pinned + n_search {
            header + idx + if has_search_sep { 1 } else { 0 }
        } else {
            header + idx
                + if has_search_sep { 1 } else { 0 }
                + if has_hist_sep { 1 } else { 0 }
        }
    }

    fn build_detail_header_lines(&self, history: &[StationData]) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let tab_active = Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [DetailTab::Overview, DetailTab::Market, DetailTab::Outfitting, DetailTab::Shipyard];
        let tab_spans: Vec<Span> = tabs.iter().flat_map(|&t| {
            let style = if t == self.detail_tab { tab_active } else { tab_inactive };
            [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
        }).collect();
        lines.push(Line::from(tab_spans));

        match self.selected_item_with_history(history) {
            Some(ListItem::Api(station)) => {
                if self.detail_tab == DetailTab::Overview {
                    lines.push(Line::from(Span::styled(
                        format!("\u{2500}\u{2500} {} \u{2500}\u{2500}", station.name),
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                    )));
                } else if self.detail_tab == DetailTab::Market {
                    lines.push(commodity_header_line(self.market_sort_col, self.market_sort_asc));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(85), Style::default().fg(Color::DarkGray))));
                } else if self.detail_tab == DetailTab::Outfitting {
                    lines.push(Line::from(Span::styled(
                        format!("{:<38} {:>12}", "Module", "Cost"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(52), Style::default().fg(Color::DarkGray))));
                } else if self.detail_tab == DetailTab::Shipyard {
                    lines.push(Line::from(Span::styled(
                        format!("{:<38} {:>14}", "Ship", "Base Value"),
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    )));
                    lines.push(Line::from(Span::styled("\u{2500}".repeat(54), Style::default().fg(Color::DarkGray))));
                }
            }
            Some(ListItem::Journal(station)) => {
                lines.push(Line::from(Span::styled(
                    format!("\u{2500}\u{2500} {} \u{2500}\u{2500} (visit snapshot)", station.name),
                    Style::default().fg(Color::Rgb(100, 180, 200)).add_modifier(Modifier::BOLD),
                )));
            }
            None => {}
        }

        lines
    }

    fn build_detail_body_lines(&self, history: &[StationData], todo_needed: &HashMap<String, i32>, ship_cargo: &HashMap<String, i32>, carrier_stock: &HashMap<String, i32>) -> Vec<Line<'static>> {
        match self.selected_item_with_history(history) {
            None => vec![Line::from(Span::styled(
                "Select a station from the list.",
                Style::default().fg(Color::DarkGray),
            ))],
            Some(ListItem::Api(station)) => match self.detail_tab {
                DetailTab::Overview => self.overview_body(station),
                DetailTab::Market => self.market_body(station, todo_needed, ship_cargo, carrier_stock),
                DetailTab::Outfitting => self.outfitting_body(station),
                DetailTab::Shipyard => self.shipyard_body(station),
            },
            Some(ListItem::Journal(station)) => match self.detail_tab {
                DetailTab::Overview => self.journal_overview_body(station),
                _ => vec![Line::from(Span::styled(
                    "No live data \u{2014} this is a visit snapshot.  Pin (p) to load full data.",
                    Style::default().fg(Color::DarkGray),
                ))],
            },
        }
    }

    fn overview_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("System:    {}", station.system_name)));
        lines.push(Line::from(format!("Type:      {}", station.station_type.as_deref().unwrap_or("Unknown"))));
        lines.push(Line::from(format!("Market ID: {}", station.market_id)));
        if let Some(ref faction) = station.faction_name { lines.push(Line::from(format!("Faction:   {faction}"))); }
        if let Some(ref gov) = station.government { lines.push(Line::from(format!("Government:{gov}"))); }
        if let Some(ref econ) = station.economy { lines.push(Line::from(format!("Economy:   {econ}"))); }
        if let Some(ref pads) = station.landing_pads {
            lines.push(Line::from(format!("Pads:      S:{} M:{} L:{}", pads.small, pads.medium, pads.large)));
        }
        if !station.services.is_empty() {
            lines.push(Line::from("Services:"));
            for chunk in station.services.chunks(3) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            format!("Updated:   {}", fmt_ts(station.updated_at.as_ref())),
            Style::default().fg(Color::DarkGray),
        )));
        lines
    }

    fn journal_overview_body(&self, station: &StationData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("System:    {}", station.system_name)));
        lines.push(Line::from(format!("Type:      {}", station.station_type)));
        lines.push(Line::from(format!("Market ID: {}", station.market_id)));
        if !station.faction.is_empty() { lines.push(Line::from(format!("Faction:   {}", station.faction))); }
        if !station.government.is_empty() { lines.push(Line::from(format!("Government:{}", station.government))); }
        if !station.allegiance.is_empty() { lines.push(Line::from(format!("Allegiance:{}", station.allegiance))); }
        if !station.economy.is_empty() {
            if station.secondary_economies.is_empty() {
                lines.push(Line::from(format!("Economy:   {}", station.economy)));
            } else {
                let secondaries: Vec<String> = station.secondary_economies.iter()
                    .map(|(name, prop)| format!("{} ({:.0}%)", name, prop * 100.0))
                    .collect();
                lines.push(Line::from(format!("Economy:   {} / {}", station.economy, secondaries.join(", "))));
            }
        }
        if let Some((s, m, l)) = station.landing_pads {
            lines.push(Line::from(format!("Pads:      S:{s} M:{m} L:{l}")));
        }
        if station.dist_from_star_ls > 0.0 {
            lines.push(Line::from(format!("Distance:  {:.0} ls", station.dist_from_star_ls)));
        }
        if !station.services.is_empty() {
            lines.push(Line::from("Services:"));
            for chunk in station.services.chunks(3) {
                lines.push(Line::from(format!("  {}", chunk.join(", "))));
            }
        }
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Visit snapshot \u{2014} pin (p) to load live market data.",
            Style::default().fg(Color::DarkGray),
        )));
        lines
    }

    fn market_body(&self, station: &StationResponse, todo_needed: &HashMap<String, i32>, ship_cargo: &HashMap<String, i32>, carrier_stock: &HashMap<String, i32>) -> Vec<Line<'static>> {
        if station.commodities.is_empty() {
            return vec![Line::from(Span::styled("No market data available.", Style::default().fg(Color::DarkGray)))];
        }
        let effective_todo: HashMap<String, i32> = station.commodities.iter()
            .filter(|c| c.buy_price > 0 && c.stock > 0)
            .filter_map(|c| {
                let norm = normalize_commodity_name(&c.name);
                todo_needed.get(&norm).map(|&n| (norm, n))
            })
            .collect();
        let mut sorted: Vec<&edcas_common::api::CommodityResponse> = station.commodities.iter().collect();
        let asc = self.market_sort_asc;
        sorted.sort_by(|a, b| {
            let a_norm = normalize_commodity_name(&a.name);
            let b_norm = normalize_commodity_name(&b.name);
            let group = effective_todo.contains_key(&b_norm).cmp(&effective_todo.contains_key(&a_norm));
            if group != Ordering::Equal { return group; }
            let col_ord = match self.market_sort_col {
                MarketSortCol::Name    => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                MarketSortCol::Buy     => a.buy_price.cmp(&b.buy_price),
                MarketSortCol::BuyDiff  => raw_diff(a.buy_price, a.mean_price).cmp(&raw_diff(b.buy_price, b.mean_price)),
                MarketSortCol::Sell    => a.sell_price.cmp(&b.sell_price),
                MarketSortCol::SellDiff => raw_diff(a.sell_price, a.mean_price).cmp(&raw_diff(b.sell_price, b.mean_price)),
                MarketSortCol::Mean    => a.mean_price.cmp(&b.mean_price),
                MarketSortCol::Stock   => a.stock.cmp(&b.stock),
                MarketSortCol::Demand  => a.demand.cmp(&b.demand),
            };
            if asc { col_ord } else { col_ord.reverse() }
        });
        let mut lines: Vec<Line<'static>> = vec![Line::from(Span::styled(
            format!("Market data as of: {}", fmt_ts(station.market_updated_at.as_ref())),
            Style::default().fg(Color::DarkGray),
        ))];
        lines.extend(sorted.into_iter().map(|c| commodity_row(c, &effective_todo, ship_cargo, carrier_stock)));
        lines
    }

    fn outfitting_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        if station.modules.is_empty() {
            return vec![Line::from(Span::styled("No outfitting data available.", Style::default().fg(Color::DarkGray)))];
        }
        let mut lines = Vec::new();
        let mut last_cat = String::new();
        for m in &station.modules {
            let cat = m.category.as_deref().unwrap_or("");
            if cat != last_cat {
                if !last_cat.is_empty() { lines.push(Line::from("")); }
                lines.push(Line::from(Span::styled(format!("[{cat}]"), Style::default().fg(Color::Yellow))));
                last_cat = cat.to_owned();
            }
            let name = m.name.as_deref().unwrap_or(&m.id);
            let cost = if m.cost > 0 { format!("{:>12}", m.cost) } else { format!("{:>12}", "-") };
            lines.push(Line::from(format!("  {:<36} {}", truncate(name, 36), cost)));
        }
        lines
    }

    fn shipyard_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        if station.ships.is_empty() {
            return vec![Line::from(Span::styled("No shipyard data available.", Style::default().fg(Color::DarkGray)))];
        }
        station.ships.iter().map(|s| {
            let name = s.name.as_deref().unwrap_or(&s.id);
            let val = if s.basevalue > 0 { format!("{:>14}", s.basevalue) } else { format!("{:>14}", "-") };
            Line::from(format!("{:<38} {}", truncate(name, 38), val))
        }).collect()
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData) -> ViewEvent {
        let history = &journal.visited_stations;
        if matches!(self.search_state, SearchState::Typing) {
            match key.code {
                KeyCode::Esc => { self.search_state = SearchState::Idle; self.status_msg = "Search cancelled".into(); }
                KeyCode::Enter => { self.search_state = SearchState::Idle; self.do_search(api); }
                KeyCode::Backspace => { self.search_query.pop(); }
                KeyCode::Char(c) => { self.search_query.push(c); }
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
                if self.display_count(history) > 0 {
                    self.toggle_pin(api, history);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => match self.focus {
                FocusArea::List => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(1); }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.focus {
                FocusArea::List => {
                    if self.selected_idx + 1 < self.display_count(history) {
                        self.selected_idx += 1;
                        self.detail_scroll = 0;
                        self.detail_tab = DetailTab::Overview;
                    }
                }
                FocusArea::Detail => { self.detail_scroll += 1; }
            },
            KeyCode::PageUp => match self.focus {
                FocusArea::List => { self.selected_idx = self.selected_idx.saturating_sub(10); }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(10); }
            },
            KeyCode::PageDown => match self.focus {
                FocusArea::List => {
                    let max = self.display_count(history).saturating_sub(1);
                    self.selected_idx = (self.selected_idx + 10).min(max);
                }
                FocusArea::Detail => { self.detail_scroll += 10; }
            },
            KeyCode::Tab => {
                match self.focus {
                    FocusArea::List => {
                        if self.display_count(history) > 0 {
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
            KeyCode::Char(c @ '1'..='8') => {
                if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Market {
                    if let Some(new_col) = MarketSortCol::from_digit(c) {
                        if new_col == self.market_sort_col {
                            self.market_sort_asc = !self.market_sort_asc;
                        } else {
                            self.market_sort_col = new_col;
                            self.market_sort_asc = true;
                        }
                        return ViewEvent::Consumed;
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(item) = self.selected_item_with_history(history) {
                    let name = match item {
                        ListItem::Api(s) => s.system_name.trim().to_string(),
                        ListItem::Journal(s) => s.system_name.trim().to_string(),
                    };
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(cb) = self.clipboard.as_mut() {
                        let _ = cb.set_text(&name);
                        self.status_msg = format!("Copied: {name}");
                    }
                }
                return ViewEvent::Consumed;
            }
            _ => {}
        }

        ViewEvent::None
    }
}

impl StationsView {
    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData, todo: &TodoList, carrier_stock: &HashMap<String, i32>) {
        let history = &journal.visited_stations;

        let mut todo_needed: HashMap<String, i32> = HashMap::new();
        for construction in &todo.construction_items {
            for res in &construction.resources {
                let remaining = (res.required_amount - res.provided_amount).max(0);
                if remaining > 0 {
                    *todo_needed.entry(normalize_commodity_name(&res.commodity_name)).or_insert(0) += remaining;
                }
            }
        }
        for item in &journal.cargo {
            let norm = normalize_commodity_name(&item.name);
            if let Some(needed) = todo_needed.get_mut(&norm) {
                *needed = (*needed - item.count).max(0);
            }
        }
        for (norm, qty) in carrier_stock {
            if let Some(needed) = todo_needed.get_mut(norm) {
                *needed = (*needed - qty).max(0);
            }
        }
        todo_needed.retain(|_, v| *v > 0);

        let ship_cargo: HashMap<String, i32> = journal.cargo.iter()
            .map(|item| (normalize_commodity_name(&item.name), item.count))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        // Left: list
        let list_lines = self.build_list_lines(history);
        let list_height = chunks[0].height.saturating_sub(2) as usize;
        let row = self.visual_row_of_selected(history);
        let list_scroll = if row + 1 >= self.list_scroll + list_height {
            (row + 2).saturating_sub(list_height)
        } else if row < self.list_scroll {
            row.saturating_sub(1)
        } else {
            self.list_scroll
        };

        frame.render_widget(
            Paragraph::new(list_lines)
                .block(Block::default().title(" Stations ").borders(Borders::ALL).border_style(
                    if self.focus == FocusArea::List { active_border } else { inactive_border },
                ))
                .scroll((list_scroll as u16, 0)),
            chunks[0],
        );

        // Right: detail
        let detail_block = Block::default()
            .title(" Station Details \u{2014} Enter to search ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail { active_border } else { inactive_border });
        let detail_inner = detail_block.inner(chunks[1]);
        frame.render_widget(detail_block, chunks[1]);

        let header_lines = self.build_detail_header_lines(history);
        let header_height = header_lines.len() as u16;
        let detail_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .split(detail_inner);

        frame.render_widget(Paragraph::new(header_lines), detail_split[0]);

        let body_lines = self.build_detail_body_lines(history, &todo_needed, &ship_cargo, carrier_stock);
        let body_height = detail_split[1].height as usize;
        let body_max_scroll = body_lines.len().saturating_sub(body_height);
        frame.render_widget(
            Paragraph::new(body_lines).scroll((self.detail_scroll.min(body_max_scroll) as u16, 0)),
            detail_split[1],
        );
    }
}

