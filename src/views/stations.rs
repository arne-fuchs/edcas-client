use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{JournalData, StationData};
use edcas_common::api::{
    CommodityPricePoint, ConstructionDepotResponse, ConstructionQuery,
    LandingPadsResponse, StationEconomyResponse, StationQuery, StationResponse,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, Paragraph},
    Frame,
};
use std::collections::{HashMap, HashSet};

use crate::api_client::ApiClient;
use crate::todo::{ConstructionTodoItem, ConstructionTodoResource, TodoList};
use crate::views::util::{
    commodity_header_line, commodity_row, compute_todo_needed, effective_todo_for_market,
    fmt_ts, normalize_commodity_name, outfitting_lines, shipyard_lines, sorted_commodities, truncate,
    FocusArea, MarketSortCol, SearchState, StationDetailTab as DetailTab,
};
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
    /// Tracked construction depot IDs — source of truth, persisted to pins.json.
    construction_tracked: HashSet<i64>,
    /// Full API data for tracked depots (cross-session fallback when not docked).
    depots: Vec<ConstructionDepotResponse>,
    /// Index of the currently selected resource row in the construction overview.
    construction_resource_idx: usize,
    #[cfg(not(target_arch = "wasm32"))]
    loading_depots: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_depots: Arc<Mutex<Option<Result<Vec<ConstructionDepotResponse>, String>>>>,
    selected_idx: usize,
    /// Market ID of the currently selected station; used to re-resolve selected_idx
    /// when visited_stations changes order (e.g., a new dock inserts at position 0).
    selected_market_id: Option<i64>,
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

    // Market commodity selection (for price history)
    market_selected_row: usize,
    /// Commodity name currently shown in the PriceHistory tab
    commodity_history_name: String,
    commodity_history_market_id: Option<i64>,
    history_buy_data: Vec<(f64, f64)>,
    history_sell_data: Vec<(f64, f64)>,
    history_time_bounds: [f64; 2],
    history_y_bounds: [f64; 2],
    history_x_labels: Vec<String>,
    history_y_labels: Vec<String>,
    #[cfg(not(target_arch = "wasm32"))]
    loading_history: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pending_history: Arc<Mutex<Option<Result<Vec<CommodityPricePoint>, String>>>>,
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
            construction_tracked: pins.constructions,
            depots: Vec::new(),
            construction_resource_idx: 0,
            #[cfg(not(target_arch = "wasm32"))]
            loading_depots: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_depots: Arc::new(Mutex::new(None)),
            selected_idx: 0,
            selected_market_id: None,
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
            market_selected_row: 0,
            commodity_history_name: String::new(),
            commodity_history_market_id: None,
            history_buy_data: Vec::new(),
            history_sell_data: Vec::new(),
            history_time_bounds: [0.0, 1.0],
            history_y_bounds: [0.0, 1.0],
            history_x_labels: Vec::new(),
            history_y_labels: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            loading_history: false,
            #[cfg(not(target_arch = "wasm32"))]
            pending_history: Arc::new(Mutex::new(None)),
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_search(&mut self) {
        if let Some(results) = self.pending_search.borrow_mut().take() {
            let count = results.len();
            self.results = results;
            self.selected_idx = 0;
            self.selected_market_id = None;
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
                Ok(api_results) => {
                    // Merge: replace/enrich stubs with API data; keep stubs for stations the API didn't return.
                    // Skip stations that were unpinned while the request was in flight.
                    for api_station in api_results {
                        if self.pinned_ids.contains(&api_station.market_id) {
                            merge_into_pinned(&mut self.pinned_results, api_station);
                        }
                    }
                    self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
                    if self.status_msg.starts_with("Loading") {
                        self.status_msg = "Press Enter to search  |  p: pin/unpin  |  c: copy system".into();
                    }
                }
                Err(e) => { self.status_msg = format!("API unavailable: {e}"); }
            }
        }
        if let Some(result) = self.pending_search.lock().unwrap().take() {
            match result {
                Ok(results) => {
                    let count = results.len();
                    self.results = results;
                    self.selected_idx = 0;
                    self.selected_market_id = None;
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

        let history_result = self.pending_history.lock().unwrap().take();
        if let Some(result) = history_result {
            self.loading_history = false;
            match result {
                Ok(points) => self.set_commodity_history_data(points),
                Err(e) => {
                    self.history_buy_data.clear();
                    self.history_sell_data.clear();
                    self.history_x_labels = vec![format!("Error: {e}")];
                }
            }
        }
    }

    pub fn on_enter(&mut self, api: &ApiClient, journal: &JournalData) {
        let history = &journal.visited_stations;
        for &mid in &self.pinned_ids {
            if !self.pinned_results.iter().any(|r| r.market_id == mid) {
                if let Some(h) = history.iter().find(|s| s.market_id == mid) {
                    self.pinned_results.push(journal_station_to_response(h));
                }
            }
        }
        self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
        #[cfg(not(target_arch = "wasm32"))]
        if !self.pinned_ids.is_empty() && !self.loading_pins {
            self.refresh_pins(api);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let has_unloaded = self.construction_tracked.iter()
                .any(|id| !self.depots.iter().any(|d| d.market_id == *id));
            if has_unloaded && !self.loading_depots {
                self.refresh_tracked(api);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_pins(&mut self, api: &ApiClient) {
        self.loading_pins = true;
        // Do NOT clear pinned_results here — stubs remain visible while API is in flight.
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

    fn save_tracked(&self) {
        let mut pins = crate::pins::Pins::load();
        pins.constructions = self.construction_tracked.clone();
        pins.save();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn refresh_tracked(&mut self, api: &ApiClient) {
        self.loading_depots = true;
        let pending = Arc::clone(&self.pending_depots);
        let api_owned = api.clone();
        let ids: Vec<i64> = self.construction_tracked.iter()
            .filter(|id| !self.depots.iter().any(|d| &&d.market_id == id))
            .copied()
            .collect();
        api.spawn(async move {
            let mut results = Vec::new();
            for mid in ids {
                let query = ConstructionQuery {
                    market_id: Some(mid),
                    limit: Some(1),
                    name: None,
                    system_name: None,
                };
                if let Ok(mut r) = api_owned.search_construction_depots(&query).await {
                    if let Some(d) = r.pop() {
                        results.push(d);
                    }
                }
            }
            *pending.lock().unwrap() = Some(Ok(results));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_construction(&mut self) {
        if let Some(result) = self.pending_depots.lock().unwrap().take() {
            self.loading_depots = false;
            if let Ok(new_depots) = result {
                for depot in new_depots {
                    if !self.depots.iter().any(|d| d.market_id == depot.market_id) {
                        self.depots.push(depot);
                    }
                }
                self.depots.sort_by(|a, b| {
                    a.system_name.cmp(&b.system_name).then(a.station_name.cmp(&b.station_name))
                });
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    pub fn poll_construction(&mut self) {}

    pub fn update_construction_from_journal(
        &mut self,
        api: &ApiClient,
        journal: &JournalData,
        todo: &mut TodoList,
    ) {
        let mut any_new = false;

        for depot in journal.construction_depots.values() {
            let mid = depot.submission.market_id;
            let existing = self.depots.iter_mut().find(|d| d.market_id == mid);

            if let Some(entry) = existing {
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
                todo.update_construction_item(ConstructionTodoItem {
                    market_id: mid,
                    station_name: depot.submission.station_name.clone(),
                    system_name: depot.system_name.clone(),
                    resources: depot.submission.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .map(|r| ConstructionTodoResource {
                            commodity_name: r.name.clone(),
                            display_name: r.display_name.clone(),
                            required_amount: r.required_amount,
                            provided_amount: r.provided_amount,
                            payment: r.payment,
                        })
                        .collect(),
                });
            } else {
                self.construction_tracked.insert(mid);
                any_new = true;

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let api_owned = api.clone();
                    let submission = depot.submission.clone();
                    api.spawn(async move {
                        let _ = api_owned.submit_construction_depot(&submission).await;
                    });
                }
                #[cfg(target_arch = "wasm32")]
                let _ = api;

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

    /// Tracks a new construction depot and triggers an API fetch for its data.
    pub fn track_construction(&mut self, market_id: i64, api: &ApiClient) {
        if self.construction_tracked.contains(&market_id) {
            return;
        }
        self.construction_tracked.insert(market_id);
        self.save_tracked();
        #[cfg(not(target_arch = "wasm32"))]
        if !self.depots.iter().any(|d| d.market_id == market_id) && !self.loading_depots {
            self.refresh_tracked(api);
        }
        #[cfg(target_arch = "wasm32")]
        let _ = api;
    }

    fn toggle_construction_todo(&mut self, market_id: i64, journal: &JournalData, todo: &mut TodoList) {
        if todo.construction_items.iter().any(|i| i.market_id == market_id) {
            todo.remove_construction_item(market_id);
        } else {
            let item = if let Some(local) = journal.construction_depots.get(&market_id) {
                super::construction::construction_todo_item_from_depot(local)
            } else if let Some(depot) = self.depots.iter().find(|d| d.market_id == market_id).cloned() {
                super::construction::construction_todo_item_from_response(&depot)
            } else {
                return;
            };
            todo.add_construction_item(item);
            if self.construction_tracked.insert(market_id) {
                self.save_tracked();
            }
        }
        todo.save();
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

    /// Called after each journal update. Stores the latest external ID sets and
    /// re-resolves selected_idx so the selection follows the same station even
    /// when visited_stations changes order.
    pub fn on_journal_update(&mut self, journal: &JournalData) {
        // Auto-unpin construction sites that are now complete.
        let completed: Vec<i64> = self.construction_tracked.iter().copied()
            .filter(|&mid| {
                journal.construction_depots.get(&mid)
                    .map(|d| d.submission.construction_complete)
                    .unwrap_or(false)
                    || self.depots.iter().any(|d| d.market_id == mid && d.construction_complete)
            })
            .collect();
        if !completed.is_empty() {
            for mid in &completed {
                self.construction_tracked.remove(mid);
                self.pinned_ids.remove(mid);
                self.pinned_results.retain(|r| r.market_id != *mid);
            }
            self.save_pins();
            self.save_tracked();
        }

        let Some(mid) = self.selected_market_id else { return; };
        let mids: Vec<i64> = self.build_display_list(journal).iter().map(|item| match item {
            ListItem::Api(s) => s.market_id,
            ListItem::Journal(s) => s.market_id,
        }).collect();
        if let Some(pos) = mids.iter().position(|&m| m == mid) {
            self.selected_idx = pos;
        } else {
            self.selected_market_id = None;
            if !mids.is_empty() {
                self.selected_idx = self.selected_idx.min(mids.len() - 1);
            }
        }
    }

    fn mid_at_idx(&self, journal: &JournalData) -> Option<i64> {
        self.selected_item(journal).map(|item| match item {
            ListItem::Api(s) => s.market_id,
            ListItem::Journal(s) => s.market_id,
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_auto_fetch(&mut self) {
        if let Some(result) = self.pending_auto_fetch.lock().unwrap().take() {
            if let Some(id) = self.auto_fetch_in_flight.take() {
                self.auto_fetch_attempted.insert(id);
            }
            if let Ok(mut results) = result {
                if let Some(station) = results.pop() {
                    let mid = station.market_id;
                    if self.pinned_ids.contains(&mid) {
                        // Already pinned — merge directly into pinned_results.
                        merge_into_pinned(&mut self.pinned_results, station);
                        self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
                    } else {
                        self.auto_fetched.insert(mid, station);
                    }
                }
            }
        }
    }

    fn build_display_list<'a>(&'a self, journal: &'a JournalData) -> Vec<ListItem<'a>> {
        let stations    = &journal.visited_stations;
        let carriers    = &journal.visited_carriers;
        let current_sys = journal.current_system.as_ref().map(|s| s.name.as_str()).unwrap_or_default();
        let mut shown: HashSet<i64> = HashSet::new();
        let mut list: Vec<ListItem<'a>> = Vec::new();

        // 1. Pinned stations
        for s in &self.pinned_results {
            shown.insert(s.market_id);
            list.push(ListItem::Api(s));
        }
        // 1b. Tracked construction sites not already pinned
        let tracked: Vec<&StationData> = stations.iter().filter(|s| {
            self.construction_tracked.contains(&s.market_id) && !shown.contains(&s.market_id)
        }).collect();
        for s in tracked {
            shown.insert(s.market_id);
            if let Some(f) = self.auto_fetched.get(&s.market_id) { list.push(ListItem::Api(f)); }
            else { list.push(ListItem::Journal(s)); }
        }

        // 2. Search results
        let search: Vec<&StationResponse> = self.results.iter().filter(|s| !shown.contains(&s.market_id)).collect();
        for s in search { shown.insert(s.market_id); list.push(ListItem::Api(s)); }

        // 3. Current system
        if !current_sys.is_empty() {
            let cur_s: Vec<&StationData> = stations.iter().filter(|s| s.system_name == current_sys && !shown.contains(&s.market_id)).collect();
            for s in cur_s {
                shown.insert(s.market_id);
                if let Some(f) = self.auto_fetched.get(&s.market_id) { list.push(ListItem::Api(f)); }
                else { list.push(ListItem::Journal(s)); }
            }
            let cur_c: Vec<&StationData> = carriers.iter().filter(|c| c.system_name == current_sys && !shown.contains(&c.market_id)).collect();
            for c in cur_c { shown.insert(c.market_id); list.push(ListItem::Journal(c)); }
        }

        // 5. History
        let hist_s: Vec<&StationData> = stations.iter().filter(|s| s.system_name != current_sys && !shown.contains(&s.market_id)).collect();
        for s in hist_s {
            shown.insert(s.market_id);
            if let Some(f) = self.auto_fetched.get(&s.market_id) { list.push(ListItem::Api(f)); }
            else { list.push(ListItem::Journal(s)); }
        }
        let hist_c: Vec<&StationData> = carriers.iter().filter(|c| c.system_name != current_sys && !shown.contains(&c.market_id)).collect();
        for c in hist_c { shown.insert(c.market_id); list.push(ListItem::Journal(c)); }

        list
    }

    fn display_count(&self, journal: &JournalData) -> usize {
        self.build_display_list(journal).len()
    }

    fn selected_item<'a>(&'a self, journal: &'a JournalData) -> Option<ListItem<'a>> {
        self.build_display_list(journal).into_iter().nth(self.selected_idx)
    }

    /// Returns `Some(market_id)` when a station is being **pinned** (not unpinned),
    /// so the caller can emit `TrackConstruction` for construction sites.
    fn toggle_pin(&mut self, api: &ApiClient, journal: &JournalData) -> Option<i64> {
        let history = &journal.visited_stations;
        let n = self.pinned_results.len();
        if self.selected_idx < n {
            // Unpin
            let mid = self.pinned_results[self.selected_idx].market_id;
            self.pinned_ids.remove(&mid);
            self.pinned_results.remove(self.selected_idx);
            let total = self.display_count(journal);
            if total > 0 {
                self.selected_idx = self.selected_idx.min(total - 1);
            } else {
                self.selected_idx = 0;
            }
            self.save_pins();
            return None;
        }
        // Clone the data from the selected item before mutating self.
        let item_data = self.build_display_list(journal).into_iter()
            .nth(self.selected_idx)
            .map(|item| match item {
                ListItem::Api(s)      => (s.market_id, Some(s.clone()), None),
                ListItem::Journal(s)  => (s.market_id, None, Some(s.clone())),
            });
        let Some((mid, api_data, journal_data)) = item_data else {
            return None;
        };
        if self.pinned_ids.contains(&mid) {
            return None;
        }
        self.pinned_ids.insert(mid);
        let stub = if let Some(s) = api_data {
            s
        } else if let Some(fetched) = self.auto_fetched.remove(&mid) {
            fetched
        } else if let Some(ref s) = journal_data {
            journal_station_to_response(s)
        } else {
            return None;
        };
        merge_into_pinned(&mut self.pinned_results, stub);
        self.pinned_results.sort_by(|a, b| a.name.cmp(&b.name));
        if let Some(pos) = self.pinned_results.iter().position(|r| r.market_id == mid) {
            self.selected_idx = pos;
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.refresh_pins(api);
        self.save_pins();
        Some(mid)
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

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_commodity_history(&mut self, api: &ApiClient) {
        let Some(market_id) = self.commodity_history_market_id else { return };
        let commodity = self.commodity_history_name.clone();
        if commodity.is_empty() { return; }
        self.loading_history = true;
        self.history_buy_data.clear();
        self.history_sell_data.clear();
        self.history_x_labels.clear();
        let pending = Arc::clone(&self.pending_history);
        let api_clone = api.clone();
        api.spawn(async move {
            let result = api_clone
                .fetch_commodity_price_history(market_id, &commodity, 30)
                .await
                .map_err(|e| e.to_string());
            *pending.lock().unwrap() = Some(result);
        });
    }

    fn set_commodity_history_data(&mut self, mut points: Vec<CommodityPricePoint>) {
        if points.is_empty() {
            self.history_buy_data.clear();
            self.history_sell_data.clear();
            self.history_x_labels = vec!["No data yet".to_string()];
            return;
        }

        points.dedup_by(|a, b| {
            let ta = a.timestamp.timestamp() / 3600;
            let tb = b.timestamp.timestamp() / 3600;
            ta == tb
        });

        let min_ts = points[0].timestamp.timestamp() as f64;
        let max_ts = points.last().unwrap().timestamp.timestamp() as f64;
        let total_days = ((max_ts - min_ts) / 86400.0).max(0.5);

        self.history_buy_data = points
            .iter()
            .map(|p| {
                let x = (p.timestamp.timestamp() as f64 - min_ts) / 86400.0;
                (x, p.buy_price as f64)
            })
            .collect();
        self.history_sell_data = points
            .iter()
            .map(|p| {
                let x = (p.timestamp.timestamp() as f64 - min_ts) / 86400.0;
                (x, p.sell_price as f64)
            })
            .collect();

        self.history_time_bounds = [0.0, total_days];

        let all_prices: Vec<f64> = points
            .iter()
            .flat_map(|p| [p.buy_price as f64, p.sell_price as f64])
            .filter(|&v| v > 0.0)
            .collect();
        let min_p = all_prices.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_p = all_prices.iter().cloned().fold(0.0_f64, f64::max);
        let padding = ((max_p - min_p) * 0.1).max(10.0);
        self.history_y_bounds = [(min_p - padding).max(0.0), max_p + padding];

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
            .map(|i| {
                let v = y_lo + (y_hi - y_lo) * i as f64 / 4.0;
                if v >= 1_000_000.0 {
                    format!("{:.1}M", v / 1_000_000.0)
                } else if v >= 1_000.0 {
                    format!("{:.0}k", v / 1_000.0)
                } else {
                    format!("{:.0}", v)
                }
            })
            .collect();
    }

    fn build_list_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let stations    = &journal.visited_stations;
        let carriers    = &journal.visited_carriers;
        let current_sys: String = journal.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default();

        let sep = |label: &str| Line::from(Span::styled(
            format!("\u{2500}\u{2500}\u{2500} {} {}", label, "\u{2500}".repeat(33usize.saturating_sub(label.len()))),
            Style::default().fg(Color::DarkGray),
        ));
        let sel_style = Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
        let build_tag = |mid: i64| {
            if self.construction_tracked.contains(&mid)
                || journal.construction_depots.contains_key(&mid)
                || self.depots.iter().any(|d| d.market_id == mid)
            { " \u{2699}" } else { "" }
        };

        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(self.search_query.clone(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(match self.search_state { SearchState::Typing => "_", SearchState::Idle => "" }, Style::default().fg(Color::Yellow)),
        ]));
        lines.push(Line::from(Span::styled(self.status_msg.clone(), Style::default().fg(Color::DarkGray))));
        lines.push(Line::from(""));

        // Track shown market IDs and the current display-list item index
        let mut shown: HashSet<i64> = HashSet::new();
        let mut item_idx = 0usize;
        let mut any_above = false;

        macro_rules! item_line {
            ($item_idx:expr, $content:expr, $default_style:expr) => {{
                let selected = self.focus == FocusArea::List && $item_idx == self.selected_idx;
                let style = if selected { sel_style } else { $default_style };
                lines.push(Line::from(Span::styled($content, style)));
                $item_idx += 1;
            }};
        }
        macro_rules! section_sep {
            ($label:expr, $n:expr, $any_above:expr) => {
                if $n > 0 && $any_above { lines.push(sep(&$label)); }
            };
        }

        // ── 1. Pinned stations ────────────────────────────────────────────────
        let n_pinned = self.pinned_results.len();
        let n_tracked_construction = stations.iter()
            .filter(|s| self.construction_tracked.contains(&s.market_id) && !self.pinned_ids.contains(&s.market_id))
            .count();
        let n_sec1 = n_pinned + n_tracked_construction;
        section_sep!("Pinned", n_sec1, any_above);
        for station in &self.pinned_results {
            shown.insert(station.market_id);
            item_line!(item_idx,
                format!(" \u{2605} {:<26} {}{}", truncate(&station.name, 26), station.station_type.as_deref().unwrap_or(""), build_tag(station.market_id)),
                Style::default().fg(Color::Yellow));
        }
        for station in stations.iter().filter(|s| self.construction_tracked.contains(&s.market_id) && !self.pinned_ids.contains(&s.market_id)) {
            shown.insert(station.market_id);
            item_line!(item_idx,
                format!(" \u{2699} {:<26} {}", truncate(&station.name, 26), &station.station_type),
                Style::default().fg(Color::Yellow));
        }
        if n_sec1 > 0 { any_above = true; }

        // ── 2. Search results ─────────────────────────────────────────────────
        let search_results: Vec<&StationResponse> = self.results.iter()
            .filter(|s| !shown.contains(&s.market_id))
            .collect();
        section_sep!("Search", search_results.len(), any_above);
        for station in &search_results {
            shown.insert(station.market_id);
            item_line!(item_idx,
                format!("   {:<26} {}{}", truncate(&station.name, 26), station.station_type.as_deref().unwrap_or(""), build_tag(station.market_id)),
                Style::default().fg(Color::White));
        }
        if !search_results.is_empty() { any_above = true; }

        // ── 3. Current system ─────────────────────────────────────────────────
        let cur_stations: Vec<&StationData> = stations.iter()
            .filter(|s| s.system_name == current_sys && !shown.contains(&s.market_id))
            .collect();
        let cur_carriers: Vec<&StationData> = carriers.iter()
            .filter(|c| c.system_name == current_sys && !shown.contains(&c.market_id))
            .collect();
        let n_current = cur_stations.len() + cur_carriers.len();
        let sys_label: &str = if current_sys.is_empty() { "Current System" } else { &current_sys };
        section_sep!(sys_label, n_current, any_above);
        for station in &cur_stations {
            shown.insert(station.market_id);
            let has_live = self.auto_fetched.contains_key(&station.market_id);
            let icon = if has_live { "\u{25CF}" } else { "  " };
            let base_style = if has_live { Style::default().fg(Color::Rgb(100, 220, 100)) } else { Style::default().fg(Color::White) };
            item_line!(item_idx,
                format!(" {} {:<26} {}{}", icon, truncate(&station.name, 26), &station.station_type, build_tag(station.market_id)),
                base_style);
        }
        for carrier in &cur_carriers {
            shown.insert(carrier.market_id);
            item_line!(item_idx,
                format!("   {:<26} {}", truncate(&carrier.name, 26), &carrier.station_type),
                Style::default().fg(Color::White));
        }
        if n_current > 0 { any_above = true; }

        // ── 5. History ────────────────────────────────────────────────────────
        let hist_stations: Vec<&StationData> = stations.iter()
            .filter(|s| s.system_name != current_sys && !shown.contains(&s.market_id))
            .collect();
        let hist_carriers: Vec<&StationData> = carriers.iter()
            .filter(|c| c.system_name != current_sys && !shown.contains(&c.market_id))
            .collect();
        let n_history = hist_stations.len() + hist_carriers.len();
        section_sep!("History", n_history, any_above);
        for station in &hist_stations {
            shown.insert(station.market_id);
            let has_live = self.auto_fetched.contains_key(&station.market_id);
            let icon = if has_live { "\u{25CF}" } else { "\u{231a}" };
            let base_style = if has_live { Style::default().fg(Color::Rgb(100, 220, 100)) } else { Style::default().fg(Color::Rgb(100, 180, 200)) };
            item_line!(item_idx,
                format!(" {} {:<26} {}{}", icon, truncate(&station.name, 26), &station.station_type, build_tag(station.market_id)),
                base_style);
        }
        for carrier in &hist_carriers {
            shown.insert(carrier.market_id);
            item_line!(item_idx,
                format!(" \u{231a} {:<26} {}", truncate(&carrier.name, 26), &carrier.system_name),
                Style::default().fg(Color::Rgb(100, 180, 200)));
        }

        if item_idx == 0 {
            lines.push(Line::from(Span::styled("No stations visited yet.", Style::default().fg(Color::DarkGray))));
        }

        lines
    }

    fn visual_row_of_selected(&self, journal: &JournalData) -> usize {
        let header = 3usize;
        let stations    = &journal.visited_stations;
        let carriers    = &journal.visited_carriers;
        let current_sys = journal.current_system.as_ref().map(|s| s.name.as_str()).unwrap_or("");
        let mut shown: HashSet<i64> = HashSet::new();

        // Compute section sizes in same order as build_display_list/build_list_lines
        let n1 = self.pinned_results.iter().map(|s| { shown.insert(s.market_id); 1usize }).sum::<usize>()
                + stations.iter().filter(|s| self.construction_tracked.contains(&s.market_id) && !self.pinned_ids.contains(&s.market_id) && shown.insert(s.market_id)).count();
        let n2 = self.results.iter().filter(|s| shown.insert(s.market_id)).count();
        let n3 = stations.iter().filter(|s| s.system_name == current_sys && shown.insert(s.market_id)).count()
               + carriers.iter().filter(|c| c.system_name == current_sys && shown.insert(c.market_id)).count();
        let n4 = stations.iter().filter(|s| s.system_name != current_sys && shown.insert(s.market_id)).count()
               + carriers.iter().filter(|c| c.system_name != current_sys && shown.insert(c.market_id)).count();

        let idx = self.selected_idx;
        let mut item_offset = 0;
        let mut sep_count = 0;
        let mut any_above = false;

        for n in [n1, n2, n3, n4] {
            if n == 0 { continue; }
            if any_above { sep_count += 1; }
            if idx < item_offset + n { return header + idx + sep_count; }
            item_offset += n;
            any_above = true;
        }

        header + idx + sep_count
    }

    fn build_detail_header_lines(&self, journal: &JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        let tab_active = Style::default().fg(Color::Black).bg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD);
        let tab_inactive = Style::default().fg(Color::Rgb(255, 140, 0));
        let tabs = [DetailTab::Overview, DetailTab::Market, DetailTab::Outfitting, DetailTab::Shipyard, DetailTab::PriceHistory];
        let tab_spans: Vec<Span> = tabs.iter().flat_map(|&t| {
            let style = if t == self.detail_tab { tab_active } else { tab_inactive };
            [Span::styled(format!(" {} ", t.label()), style), Span::raw("  ")]
        }).collect();
        lines.push(Line::from(tab_spans));

        match self.selected_item(journal) {
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
                } else if self.detail_tab == DetailTab::PriceHistory {
                    let label = if self.commodity_history_name.is_empty() {
                        "Price History — select a commodity in Market tab and press h".to_string()
                    } else {
                        format!("Price History: {}", self.commodity_history_name)
                    };
                    lines.push(Line::from(Span::styled(
                        label,
                        Style::default().fg(Color::Rgb(255, 140, 0)).add_modifier(Modifier::BOLD),
                    )));
                }
            }
            Some(ListItem::Journal(station)) => {
                let docked_here = journal.last_docked
                    .as_ref()
                    .map(|(mid, _, _, _)| *mid == station.market_id)
                    .unwrap_or(false);
                let label = if docked_here { "docked" } else { "visit snapshot" };
                lines.push(Line::from(Span::styled(
                    format!("\u{2500}\u{2500} {} \u{2500}\u{2500} ({})", station.name, label),
                    Style::default().fg(Color::Rgb(100, 180, 200)).add_modifier(Modifier::BOLD),
                )));
            }
            None => {}
        }

        lines
    }

    fn build_detail_body_lines(&self, journal: &crate::journal_reader::JournalData, todo_needed: &HashMap<String, i32>, ship_cargo: &HashMap<String, i32>) -> Vec<Line<'static>> {
        match self.selected_item(journal) {
            None => vec![Line::from(Span::styled(
                "Select a station from the list.",
                Style::default().fg(Color::DarkGray),
            ))],
            Some(ListItem::Api(station)) => match self.detail_tab {
                DetailTab::Overview => self.overview_body(station, journal),
                DetailTab::Market => self.market_body(station, todo_needed, ship_cargo),
                DetailTab::Outfitting => self.outfitting_body(station),
                DetailTab::Shipyard => self.shipyard_body(station),
                DetailTab::PriceHistory => vec![], // rendered as Chart in render()
            },
            Some(ListItem::Journal(station)) => match self.detail_tab {
                DetailTab::Overview => self.journal_overview_body(station, journal),
                DetailTab::Market => {
                    if !station.commodities.is_empty() {
                        return self.local_market_body(&station.commodities, todo_needed, ship_cargo);
                    }
                    // Fallback: check if local_market still matches this station
                    // (covers the brief window before the watcher loop stores it into the station entry)
                    if let Some((mid, ref commodities)) = journal.local_market {
                        if mid == station.market_id {
                            return self.local_market_body(commodities, todo_needed, ship_cargo);
                        }
                    }
                    let is_docked_here = journal.last_docked
                        .as_ref()
                        .map(|(mid, _, _, _)| *mid == station.market_id)
                        .unwrap_or(false);
                    if is_docked_here {
                        vec![
                            Line::from(Span::styled(
                                "No market data for this station yet.",
                                Style::default().fg(Color::DarkGray),
                            )),
                            Line::from(""),
                            Line::from(Span::styled(
                                "Open the Commodities Market panel in-game and the data will appear here automatically.",
                                Style::default().fg(Color::Rgb(180, 180, 180)),
                            )),
                        ]
                    } else {
                        vec![Line::from(Span::styled(
                            "No live data \u{2014} this is a visit snapshot.  Pin (p) to load full data.",
                            Style::default().fg(Color::DarkGray),
                        ))]
                    }
                }
                DetailTab::PriceHistory => vec![],
                _ => vec![Line::from(Span::styled(
                    "No live data \u{2014} this is a visit snapshot.  Pin (p) to load full data.",
                    Style::default().fg(Color::DarkGray),
                ))],
            },
        }
    }

    fn construction_body(&self, market_id: i64, journal: &crate::journal_reader::JournalData, selected_idx: Option<usize>) -> Vec<Line<'static>> {
        let ship_cargo: HashMap<String, i32> = journal.cargo.iter()
            .map(|item| (normalize_commodity_name(&item.name), item.count))
            .collect();
        if let Some(depot) = journal.construction_depots.get(&market_id) {
            match selected_idx {
                Some(idx) => super::construction::depot_detail_lines_local_with_selection(depot, &ship_cargo, idx),
                None => super::construction::depot_detail_lines_local(depot, &ship_cargo),
            }
        } else if let Some(depot) = self.depots.iter().find(|d| d.market_id == market_id) {
            let mut lines = Vec::new();
            super::construction::build_detail_from_response(&mut lines, depot, &ship_cargo, selected_idx);
            lines
        } else {
            vec![
                Line::from(Span::styled(
                    "No construction data for this site.",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(Span::styled(
                    "Dock at the construction site to load resource requirements.",
                    Style::default().fg(Color::Rgb(180, 180, 180)),
                )),
            ]
        }
    }

    /// Returns the market ID of the selected station if it has construction data.
    fn selected_construction_market_id(&self, journal: &JournalData) -> Option<i64> {
        let mid = self.mid_at_idx(journal)?;
        if journal.construction_depots.contains_key(&mid) || self.depots.iter().any(|d| d.market_id == mid) {
            Some(mid)
        } else {
            None
        }
    }

    fn construction_resource_count(&self, market_id: i64, journal: &crate::journal_reader::JournalData) -> usize {
        if let Some(depot) = journal.construction_depots.get(&market_id) {
            super::construction::depot_resource_count_local(depot)
        } else if let Some(depot) = self.depots.iter().find(|d| d.market_id == market_id) {
            super::construction::depot_resource_count_api(depot)
        } else {
            0
        }
    }

    fn overview_body(&self, station: &StationResponse, journal: &crate::journal_reader::JournalData) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        lines.push(Line::from(format!("System:    {}", station.system_name)));
        lines.push(Line::from(format!("Type:      {}", station.station_type.as_deref().unwrap_or("Unknown"))));
        lines.push(Line::from(format!("Market ID: {}", station.market_id)));
        if let Some(dist) = station.dist_from_star_ls.filter(|&d| d > 0.0) {
            lines.push(Line::from(format!("Distance:  {dist:.0} ls")));
        }
        let faction = station.faction_name.as_deref().or_else(|| {
            journal.visited_stations.iter()
                .find(|s| s.market_id == station.market_id)
                .map(|s| s.faction.as_str())
                .filter(|f| !f.is_empty())
        });
        if let Some(faction) = faction { lines.push(Line::from(format!("Faction:   {faction}"))); }
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
        let has_construction = journal.construction_depots.contains_key(&station.market_id)
            || self.depots.iter().any(|d| d.market_id == station.market_id);
        if has_construction {
            let sel = if self.focus == FocusArea::Detail { Some(self.construction_resource_idx) } else { None };
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("── Construction (w/s: navigate  f: search nearest  g: best station  t: todo) ──", Style::default().fg(Color::Rgb(255, 140, 0)))));
            lines.extend(self.construction_body(station.market_id, journal, sel));
        }
        lines
    }

    fn journal_overview_body(&self, station: &StationData, journal: &crate::journal_reader::JournalData) -> Vec<Line<'static>> {
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
        let has_construction = journal.construction_depots.contains_key(&station.market_id)
            || self.depots.iter().any(|d| d.market_id == station.market_id);
        if has_construction {
            let sel = if self.focus == FocusArea::Detail { Some(self.construction_resource_idx) } else { None };
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled("── Construction (w/s: navigate  f: search nearest  g: best station  t: todo) ──", Style::default().fg(Color::Rgb(255, 140, 0)))));
            lines.extend(self.construction_body(station.market_id, journal, sel));
        }
        lines
    }

    fn market_body(&self, station: &StationResponse, todo_needed: &HashMap<String, i32>, ship_cargo: &HashMap<String, i32>) -> Vec<Line<'static>> {
        if station.commodities.is_empty() {
            return vec![Line::from(Span::styled("No market data available.", Style::default().fg(Color::DarkGray)))];
        }
        let effective_todo = effective_todo_for_market(&station.commodities, todo_needed);
        let sorted = sorted_commodities(&station.commodities, &effective_todo, self.market_sort_col, self.market_sort_asc);
        let mut lines = vec![Line::from(Span::styled(
            format!("Market data as of: {} — w/s: select  h: price history", fmt_ts(station.market_updated_at.as_ref())),
            Style::default().fg(Color::DarkGray),
        ))];
        let in_detail = self.focus == FocusArea::Detail;
        for (i, c) in sorted.into_iter().enumerate() {
            let row = commodity_row(c, &effective_todo, ship_cargo);
            if in_detail && i == self.market_selected_row {
                let highlighted: Vec<Span<'static>> = row
                    .spans
                    .into_iter()
                    .map(|s| Span::styled(s.content, s.style.bg(Color::Rgb(30, 40, 55))))
                    .collect();
                lines.push(Line::from(highlighted));
            } else {
                lines.push(row);
            }
        }
        lines
    }

    fn local_market_body(&self, commodities: &[edcas_common::api::CommodityResponse], todo_needed: &HashMap<String, i32>, ship_cargo: &HashMap<String, i32>) -> Vec<Line<'static>> {
        let effective_todo = effective_todo_for_market(commodities, todo_needed);
        let sorted = sorted_commodities(commodities, &effective_todo, self.market_sort_col, self.market_sort_asc);
        let mut lines = vec![Line::from(Span::styled(
            "Local market data (from Market.json)",
            Style::default().fg(Color::DarkGray),
        ))];
        lines.extend(sorted.into_iter().map(|c| commodity_row(c, &effective_todo, ship_cargo)));
        lines
    }

    fn outfitting_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        outfitting_lines(&station.modules)
    }

    fn shipyard_body(&self, station: &StationResponse) -> Vec<Line<'static>> {
        shipyard_lines(&station.ships)
    }

    pub fn handle_key(&mut self, key: &KeyEvent, api: &ApiClient, journal: &JournalData, todo: &mut TodoList) -> ViewEvent {
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
            // In detail+overview on a construction site, f opens search nearest.
            KeyCode::Char('f') if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Overview
                && self.selected_construction_market_id(journal).is_some() =>
            {
                let mid = self.selected_construction_market_id(journal).unwrap();
                let resource = if let Some(local) = journal.construction_depots.get(&mid) {
                    let mut sorted = local.submission.resources.clone();
                    sorted.sort_by(|a, b| {
                        let done_a = a.provided_amount >= a.required_amount;
                        let done_b = b.provided_amount >= b.required_amount;
                        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
                    });
                    sorted.into_iter().nth(self.construction_resource_idx)
                        .map(|r| (r.display_name.clone(), r.name.clone()))
                } else if let Some(depot) = self.depots.iter().find(|d| d.market_id == mid).cloned() {
                    let mut sorted = depot.resources.clone();
                    sorted.sort_by(|a, b| {
                        let done_a = a.provided_amount >= a.required_amount;
                        let done_b = b.provided_amount >= b.required_amount;
                        done_a.cmp(&done_b).then(a.display_name.cmp(&b.display_name))
                    });
                    sorted.into_iter().nth(self.construction_resource_idx)
                        .map(|r| (r.display_name.clone(), r.name.clone()))
                } else {
                    None
                };
                if let Some((commodity, raw_name)) = resource {
                    let canonical_name = super::search_nearest::resolve_commodity_canonical(&raw_name);
                    let system = journal.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                    let ship_pad_size = journal.pilot.ship_pad_size;
                    return ViewEvent::OpenSearchNearest { commodity, canonical_name, system, ship_pad_size };
                }
                return ViewEvent::Consumed;
            }
            // 'g' — find the single best station covering the most missing commodities.
            KeyCode::Char('g') if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Overview
                && self.selected_construction_market_id(journal).is_some() =>
            {
                let mid = self.selected_construction_market_id(journal).unwrap();
                let commodities: Vec<String> = if let Some(local) = journal.construction_depots.get(&mid) {
                    local.submission.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .map(|r| r.display_name.clone())
                        .collect()
                } else if let Some(depot) = self.depots.iter().find(|d| d.market_id == mid) {
                    depot.resources.iter()
                        .filter(|r| r.provided_amount < r.required_amount)
                        .map(|r| r.display_name.clone())
                        .collect()
                } else {
                    vec![]
                };
                if !commodities.is_empty() {
                    let system = journal.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                    let ship_pad_size = journal.pilot.ship_pad_size;
                    return ViewEvent::OpenMultiSearch { commodities, system, ship_pad_size };
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('/') | KeyCode::Char('f') => {
                self.search_query.clear();
                self.search_state = SearchState::Typing;
                self.status_msg = "Typing… (Enter to search, Esc to cancel)".into();
            }
            KeyCode::Char('p') => {
                if self.display_count(journal) > 0 {
                    if let Some(pinned_mid) = self.toggle_pin(api, journal) {
                        if journal.construction_depots.contains_key(&pinned_mid) {
                            return ViewEvent::TrackConstruction(pinned_mid);
                        }
                    }
                }
                return ViewEvent::Consumed;
            }
            KeyCode::Char('w') | KeyCode::Up => match self.focus {
                FocusArea::List => {
                    if self.selected_idx > 0 {
                        self.selected_idx -= 1;
                        self.detail_scroll = 0;
                        self.construction_resource_idx = 0;
                        self.market_selected_row = 0;
                        self.detail_tab = DetailTab::Overview;
                        self.selected_market_id = self.mid_at_idx(journal);
                    }
                }
                FocusArea::Detail => {
                    if self.detail_tab == DetailTab::Market {
                        self.market_selected_row = self.market_selected_row.saturating_sub(1);
                        let scroll = self.market_selected_row.saturating_sub(3);
                        self.detail_scroll = self.detail_scroll.min(scroll + 1).max(scroll.saturating_sub(1));
                        return ViewEvent::Consumed;
                    } else if self.detail_tab == DetailTab::Overview && self.selected_construction_market_id(journal).is_some() {
                        self.construction_resource_idx = self.construction_resource_idx.saturating_sub(1);
                    } else {
                        self.detail_scroll = self.detail_scroll.saturating_sub(1);
                    }
                }
            },
            KeyCode::Char('s') | KeyCode::Down => match self.focus {
                FocusArea::List => {
                    if self.selected_idx + 1 < self.display_count(journal) {
                        self.selected_idx += 1;
                        self.detail_scroll = 0;
                        self.construction_resource_idx = 0;
                        self.market_selected_row = 0;
                        self.detail_tab = DetailTab::Overview;
                        self.selected_market_id = self.mid_at_idx(journal);
                    }
                }
                FocusArea::Detail => {
                    if self.detail_tab == DetailTab::Market {
                        let max_rows = self.selected_item(journal)
                            .and_then(|item| if let ListItem::Api(s) = item { Some(s.commodities.len().saturating_sub(1)) } else { None })
                            .unwrap_or(0);
                        if self.market_selected_row < max_rows {
                            self.market_selected_row += 1;
                        }
                        return ViewEvent::Consumed;
                    } else if self.detail_tab == DetailTab::Overview {
                        if let Some(mid) = self.selected_construction_market_id(journal) {
                            let max = self.construction_resource_count(mid, journal).saturating_sub(1);
                            if self.construction_resource_idx < max {
                                self.construction_resource_idx += 1;
                            }
                            return ViewEvent::Consumed;
                        }
                    }
                    self.detail_scroll += 1;
                }
            },
            KeyCode::Char('h') if self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Market => {
                let history_target = if let Some(ListItem::Api(station)) = self.selected_item(journal) {
                    let effective_todo = compute_todo_needed(&todo.construction_items, &journal.cargo);
                    let local_effective = effective_todo_for_market(&station.commodities, &effective_todo);
                    let sorted = sorted_commodities(&station.commodities, &local_effective, self.market_sort_col, self.market_sort_asc);
                    sorted.get(self.market_selected_row).map(|c| (c.name.clone(), station.market_id))
                } else {
                    None
                };
                if let Some((name, market_id)) = history_target {
                    self.commodity_history_name = name;
                    self.commodity_history_market_id = Some(market_id);
                    self.detail_tab = DetailTab::PriceHistory;
                    self.detail_scroll = 0;
                    #[cfg(not(target_arch = "wasm32"))]
                    self.fetch_commodity_history(api);
                }
                return ViewEvent::Consumed;
            }
            KeyCode::PageUp => match self.focus {
                FocusArea::List => {
                    self.selected_idx = self.selected_idx.saturating_sub(10);
                    self.selected_market_id = self.mid_at_idx(journal);
                }
                FocusArea::Detail => { self.detail_scroll = self.detail_scroll.saturating_sub(10); }
            },
            KeyCode::PageDown => match self.focus {
                FocusArea::List => {
                    let max = self.display_count(journal).saturating_sub(1);
                    self.selected_idx = (self.selected_idx + 10).min(max);
                    self.selected_market_id = self.mid_at_idx(journal);
                }
                FocusArea::Detail => { self.detail_scroll += 10; }
            },
            KeyCode::Tab => {
                match self.focus {
                    FocusArea::List => {
                        if self.display_count(journal) > 0 {
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
            KeyCode::Char('t') => {
                let ok_tab = self.focus == FocusArea::List
                    || (self.focus == FocusArea::Detail && self.detail_tab == DetailTab::Overview);
                if ok_tab {
                    let mid = self.mid_at_idx(journal);
                    let is_construction = mid.map(|m| {
                        self.construction_tracked.contains(&m)
                            || journal.construction_depots.contains_key(&m)
                            || self.depots.iter().any(|d| d.market_id == m)
                    }).unwrap_or(false);
                    if is_construction {
                        if let Some(mid) = self.selected_construction_market_id(journal) {
                            self.toggle_construction_todo(mid, journal, todo);
                        } else {
                            self.status_msg = "No resource data yet — dock at the construction depot first".into();
                        }
                        return ViewEvent::Consumed;
                    }
                }
            }
            KeyCode::Char('c') => {
                if let Some(item) = self.selected_item(journal) {
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

fn journal_station_to_response(s: &StationData) -> StationResponse {
    StationResponse {
        market_id: s.market_id,
        system_address: 0,
        system_name: s.system_name.clone(),
        name: s.name.clone(),
        station_type: Some(s.station_type.clone()),
        faction_name: if s.faction.is_empty() { None } else { Some(s.faction.clone()) },
        government: if s.government.is_empty() { None } else { Some(s.government.clone()) },
        economy: if s.economy.is_empty() { None } else { Some(s.economy.clone()) },
        economies: s.secondary_economies.iter()
            .map(|(name, prop)| StationEconomyResponse { name: name.clone(), proportion: *prop })
            .collect(),
        services: s.services.clone(),
        landing_pads: s.landing_pads.map(|(small, medium, large)| LandingPadsResponse { small, medium, large }),
        dist_from_star_ls: if s.dist_from_star_ls > 0.0 { Some(s.dist_from_star_ls) } else { None },
        carrier_name: None,
        updated_at: None,
        market_updated_at: None,
        commodities: s.commodities.clone(),
        modules: Vec::new(),
        ships: Vec::new(),
    }
}

fn merge_into_pinned(pinned: &mut Vec<StationResponse>, api_station: StationResponse) {
    if let Some(existing) = pinned.iter_mut().find(|r| r.market_id == api_station.market_id) {
        let keep_commodities = if api_station.commodities.is_empty() {
            existing.commodities.clone()
        } else {
            api_station.commodities.clone()
        };
        *existing = api_station;
        existing.commodities = keep_commodities;
    } else {
        pinned.push(api_station);
    }
}

impl StationsView {
    pub fn render(&self, frame: &mut Frame, area: Rect, journal: &JournalData, todo: &TodoList) {
        let todo_needed = compute_todo_needed(&todo.construction_items, &journal.cargo);
        let ship_cargo: HashMap<String, i32> = journal.cargo.iter()
            .map(|item| (normalize_commodity_name(&item.name), item.count))
            .collect();

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
            .split(area);

        let active_border = Style::default().fg(Color::Rgb(255, 140, 0));
        let inactive_border = Style::default().fg(Color::White);

        let list_lines = self.build_list_lines(journal);
        let list_height = chunks[0].height.saturating_sub(2) as usize;
        let row = self.visual_row_of_selected(journal);
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

        let detail_block = Block::default()
            .title(" Station Details \u{2014} / or f: search ")
            .borders(Borders::ALL)
            .border_style(if self.focus == FocusArea::Detail { active_border } else { inactive_border });
        let detail_inner = detail_block.inner(chunks[1]);
        frame.render_widget(detail_block, chunks[1]);

        let header_lines = self.build_detail_header_lines(journal);
        let header_height = header_lines.len() as u16;
        let detail_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(header_height), Constraint::Min(0)])
            .split(detail_inner);

        frame.render_widget(Paragraph::new(header_lines), detail_split[0]);

        if self.detail_tab == DetailTab::PriceHistory {
            self.render_price_history_chart(frame, detail_split[1]);
        } else {
            let body_lines = self.build_detail_body_lines(journal, &todo_needed, &ship_cargo);
            let body_height = detail_split[1].height as usize;
            let body_max_scroll = body_lines.len().saturating_sub(body_height);
            frame.render_widget(
                Paragraph::new(body_lines).scroll((self.detail_scroll.min(body_max_scroll) as u16, 0)),
                detail_split[1],
            );
        }
    }

    fn render_price_history_chart(&self, frame: &mut Frame, area: Rect) {
        #[cfg(not(target_arch = "wasm32"))]
        if self.loading_history {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "Loading price history…",
                    Style::default().fg(Color::DarkGray),
                ))),
                area,
            );
            return;
        }

        if self.commodity_history_name.is_empty() {
            frame.render_widget(
                Paragraph::new(Line::from(Span::styled(
                    "Go to Market tab, select a commodity with w/s, then press h.",
                    Style::default().fg(Color::DarkGray),
                ))),
                area,
            );
            return;
        }

        if self.history_buy_data.len() < 2 {
            let msg = if self.history_buy_data.is_empty() {
                "No price history recorded yet for this commodity."
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

        let buy_dataset = Dataset::default()
            .name("Buy price")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Rgb(255, 140, 0)))
            .data(&self.history_buy_data);

        let sell_dataset = Dataset::default()
            .name("Sell price")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Green))
            .data(&self.history_sell_data);

        let chart = Chart::new(vec![buy_dataset, sell_dataset])
            .x_axis(
                Axis::default()
                    .title("Date")
                    .bounds(self.history_time_bounds)
                    .labels(x_labels)
                    .style(Style::default().fg(Color::DarkGray)),
            )
            .y_axis(
                Axis::default()
                    .title("Credits")
                    .bounds(self.history_y_bounds)
                    .labels(y_labels)
                    .style(Style::default().fg(Color::DarkGray)),
            );

        frame.render_widget(chart, area);
    }
}

