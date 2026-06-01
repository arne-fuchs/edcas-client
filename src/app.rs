use std::path::PathBuf;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};
use tracing::{info, warn};

use crate::api_client::ApiClient;
use crate::event_shim::{KeyCode, KeyEvent};
use crate::journal_reader::{BodyMaterial, BodyParent, BodyRing, BodyScan, JournalData, ParentType};
use crate::settings::Settings;
use crate::views::{
    AboutView, CarriersView, ConstructionView, EngineersView, ExplorerView, FactionsView,
    InventoryView, ModulesView, NewsView, PilotView, SearchNearestView, SettingsView, StationsView,
    TodoView, TradeRoutesView, ViewEvent,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::journal_reader::JournalReader;

pub const APP_TITLE: &str = "EDCAS - Elite Dangerous Commander Assistant System";

pub const TABS: &[&str] = &[
    "News",
    "Explorer",
    "Stations",
    "Carriers",
    "Factions",
    "Construction",
    "Trade Routes",
    "Pilot",
    "Ship",
    "Inventory",
    "Engineers",
    "Todo",
    "Search Nearest",
    "Settings",
    "About",
];

#[derive(Default, Clone, Copy, PartialEq)]
pub enum AppView {
    #[default]
    News = 0,
    Explorer = 1,
    Stations = 2,
    Carriers = 3,
    Factions = 4,
    Construction = 5,
    TradeRoutes = 6,
    Pilot = 7,
    Modules = 8,
    Materials = 9,
    Engineers = 10,
    Todo = 11,
    SearchNearest = 12,
    Settings = 13,
    About = 14,
}

impl AppView {
    pub fn next(&self) -> Self {
        match self {
            AppView::News => AppView::Explorer,
            AppView::Explorer => AppView::Stations,
            AppView::Stations => AppView::Carriers,
            AppView::Carriers => AppView::Factions,
            AppView::Factions => AppView::Construction,
            AppView::Construction => AppView::TradeRoutes,
            AppView::TradeRoutes => AppView::Pilot,
            AppView::Pilot => AppView::Modules,
            AppView::Modules => AppView::Materials,
            AppView::Materials => AppView::Engineers,
            AppView::Engineers => AppView::Todo,
            AppView::Todo => AppView::SearchNearest,
            AppView::SearchNearest => AppView::Settings,
            AppView::Settings => AppView::About,
            AppView::About => AppView::News,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            AppView::News => AppView::About,
            AppView::Explorer => AppView::News,
            AppView::Stations => AppView::Explorer,
            AppView::Carriers => AppView::Stations,
            AppView::Factions => AppView::Carriers,
            AppView::Construction => AppView::Factions,
            AppView::TradeRoutes => AppView::Construction,
            AppView::Pilot => AppView::TradeRoutes,
            AppView::Modules => AppView::Pilot,
            AppView::Materials => AppView::Modules,
            AppView::Engineers => AppView::Materials,
            AppView::Todo => AppView::Engineers,
            AppView::SearchNearest => AppView::Todo,
            AppView::Settings => AppView::SearchNearest,
            AppView::About => AppView::Settings,
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }
}

pub struct App {
    pub view: AppView,
    pub settings: Settings,
    pub journal: JournalData,
    #[cfg(not(target_arch = "wasm32"))]
    pub journal_reader: Option<JournalReader>,
    /// Owns the tokio runtime — must outlive `api` which holds a `Handle`.
    #[cfg(not(target_arch = "wasm32"))]
    _rt: tokio::runtime::Runtime,
    pub api: ApiClient,
    #[cfg(not(target_arch = "wasm32"))]
    pub api_rx: Option<std::sync::mpsc::Receiver<Vec<edcas_common::api::BodyResponse>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub last_api_system: i64,
    /// Bodies fetched from the server API for the current system.
    /// Re-merged on every journal update so they aren't lost when the watcher
    /// overwrites `self.journal` with a fresh data snapshot.
    #[cfg(not(target_arch = "wasm32"))]
    cached_api_bodies: Vec<BodyScan>,
    /// Receives the next predicted BGS tick from the background refresh task.
    #[cfg(not(target_arch = "wasm32"))]
    pending_tick: std::sync::Arc<std::sync::Mutex<Option<chrono::DateTime<chrono::Utc>>>>,
    pub news: NewsView,
    pub pilot: PilotView,
    pub explorer: ExplorerView,
    pub inventory: InventoryView,
    pub modules_view: ModulesView,
    pub stations: StationsView,
    pub carriers: CarriersView,
    pub factions: FactionsView,
    pub construction: ConstructionView,
    pub trade_routes: TradeRoutesView,
    pub engineers_view: EngineersView,
    pub todo_view: TodoView,
    pub search_nearest: SearchNearestView,
    pub settings_view: SettingsView,
    pub about: AboutView,
    pub should_quit: bool,
    pub next_tick: Option<chrono::DateTime<chrono::Utc>>,
    my_carrier_data: crate::my_carriers::MyCarriersData,
    #[cfg(not(target_arch = "wasm32"))]
    last_docked_market_id: Option<i64>,
}

impl App {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Self {
        let settings = Settings::default();
        info!("Settings loaded");

        let journal_dir = if !settings.journal_reader.journal_directory.is_empty() {
            let dir = PathBuf::from(&settings.journal_reader.journal_directory);
            info!("Journal directory configured: {}", dir.display());
            Some(dir)
        } else {
            warn!("No journal directory configured");
            None
        };

        let journal = JournalData::new();

        let api_url = settings.api_url.trim().to_string();
        let journal_reader = journal_dir.map(|dir| {
            info!("Starting journal reader for directory: {}", dir.display());
            let url = if api_url.is_empty() { None } else { Some(api_url.clone()) };
            JournalReader::start(dir, url)
        });

        let rt = tokio::runtime::Runtime::new().expect("failed to build async runtime");
        let api = ApiClient::new(&settings.api_url, rt.handle().clone());

        let mut app = Self {
            view: AppView::default(),
            settings,
            journal,
            journal_reader,
            _rt: rt,
            api,
            api_rx: None,
            last_api_system: 0,
            cached_api_bodies: Vec::new(),
            pending_tick: std::sync::Arc::new(std::sync::Mutex::new(None)),
            news: NewsView::new(),
            pilot: PilotView::new(),
            explorer: ExplorerView::new(),
            inventory: InventoryView::new(),
            modules_view: ModulesView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            factions: FactionsView::new(),
            construction: ConstructionView::new(),
            trade_routes: TradeRoutesView::new(),
            engineers_view: EngineersView::new(),
            todo_view: TodoView::new(),
            search_nearest: SearchNearestView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
            next_tick: None,
            my_carrier_data: crate::my_carriers::MyCarriersData::load(),
            last_docked_market_id: None,
        };
        app.news.start_fetch(&app.api);
        app.refresh_server_tick();
        app
    }

    #[cfg(target_arch = "wasm32")]
    pub fn new_web() -> Self {
        let settings = Settings::default();
        let api = ApiClient::new(&settings.api_url);
        Self {
            view: AppView::default(),
            settings,
            journal: JournalData::new(),
            api,
            news: NewsView::new(),
            pilot: PilotView::new(),
            explorer: ExplorerView::new(),
            inventory: InventoryView::new(),
            modules_view: ModulesView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            factions: FactionsView::new(),
            construction: ConstructionView::new(),
            trade_routes: TradeRoutesView::new(),
            engineers_view: EngineersView::new(),
            todo_view: TodoView::new(),
            search_nearest: SearchNearestView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
            next_tick: None,
            my_carrier_data: crate::my_carriers::MyCarriersData::load(),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_journal_updates(&mut self) {
        // Drain into a Vec first so the borrow on `self.journal_reader` is released
        // before we call methods that need full `&mut self`.
        let updates: Vec<_> = match self.journal_reader {
            Some(ref reader) => {
                let mut v = Vec::new();
                while let Some(data) = reader.try_recv() {
                    v.push(data);
                }
                v
            }
            None => Vec::new(),
        };

        for data in updates {
            let system_name = data.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default();
            let body_count = data.bodies.len();
            info!("Journal update received: system={}, bodies={}", system_name, body_count);

            let new_addr = data.current_system.as_ref().map(|s| s.system_address).unwrap_or(0);
            if new_addr != 0 && new_addr != self.last_api_system && !self.settings.api_url.is_empty() {
                self.last_api_system = new_addr;
                self.cached_api_bodies.clear();
                let (tx, rx) = std::sync::mpsc::channel();
                self.api_rx = Some(rx);
                let api = self.api.clone();
                info!("Fetching bodies from API for system {}", new_addr);
                self.api.spawn(async move {
                    match api.get_bodies(new_addr).await {
                        Ok(bodies) => {
                            info!("API returned {} bodies for system {}", bodies.len(), new_addr);
                            let _ = tx.send(bodies);
                        }
                        Err(e) => {
                            warn!("API bodies fetch failed for system {}: {:#}", new_addr, e);
                        }
                    }
                });
            }

            self.journal = data;
            self.persist_my_carrier_cargo();

            // Keep the stations view selection on the same station even when
            // visited_stations changes order (e.g., a new dock prepends a station).
            self.stations.on_journal_update(&self.journal.visited_stations);

            // Auto-fetch full market data when docking at a non-carrier station.
            let new_dock = self.journal.last_docked.as_ref().map(|(mid, _, _, _)| *mid);
            if new_dock != self.last_docked_market_id {
                self.last_docked_market_id = new_dock;
                if let Some(market_id) = new_dock {
                    let is_carrier = self.journal.visited_carriers.iter().any(|c| c.market_id == market_id);
                    if !is_carrier && !self.settings.api_url.is_empty() {
                        self.stations.fetch_on_dock(market_id, &self.api);
                    }
                }
            }

            // Re-merge API bodies: the watcher snapshot doesn't include them,
            // so without this they're lost on every subsequent journal update.
            let local_ids: std::collections::HashSet<i32> =
                self.journal.bodies.iter().map(|b| b.body_id).collect();
            for body in &self.cached_api_bodies {
                if !local_ids.contains(&body.body_id) {
                    self.journal.bodies.push(body.clone());
                }
            }
            self.explorer.update(&self.journal);

            self.construction.update_from_journal(&self.api, &self.journal, &mut self.todo_view.todo);
        }

        self.trade_routes.poll_results();

        if let Some(ref rx) = self.api_rx {
            if let Ok(api_bodies) = rx.try_recv() {
                self.api_rx = None;
                let local_ids: std::collections::HashSet<i32> =
                    self.journal.bodies.iter().map(|b| b.body_id).collect();
                let mut added = 0usize;
                for br in &api_bodies {
                    if !local_ids.contains(&br.id) {
                        let body = body_from_api(br);
                        self.cached_api_bodies.push(body.clone());
                        self.journal.bodies.push(body);
                        added += 1;
                    }
                }
                if added > 0 {
                    info!("Merged {} bodies from API into explorer", added);
                    self.explorer.update(&self.journal);
                } else {
                    info!("API returned {} bodies, all already present locally", api_bodies.len());
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn persist_my_carrier_cargo(&mut self) {
        let my_ids: Vec<i64> = self.carriers.my_carrier_ids().iter().copied().collect();
        let mut changed = false;

        // Remove carriers no longer marked as mine
        self.my_carrier_data.carriers.retain(|id, _| {
            if !my_ids.contains(id) {
                changed = true;
                false
            } else {
                true
            }
        });

        for &id in &my_ids {
            if let Some(live) = self.journal.carrier_cargo.get(&id) {
                let entry = self.my_carrier_data.carriers.entry(id).or_insert_with(|| {
                    changed = true;
                    std::collections::HashMap::new()
                });
                if entry != live {
                    *entry = live.clone();
                    changed = true;
                }
            } else {
                // No live data this session; ensure the key exists to mark carrier as mine
                if !self.my_carrier_data.carriers.contains_key(&id) {
                    self.my_carrier_data.carriers.insert(id, std::collections::HashMap::new());
                    changed = true;
                }
            }
        }

        if changed {
            // Record the latest journal event timestamp so that on a mid-session restart
            // we can skip CargoTransfer events already captured in this snapshot.
            if !self.journal.latest_event_timestamp.is_empty() {
                self.my_carrier_data.snapshot_timestamp = self.journal.latest_event_timestamp.clone();
            }
            self.my_carrier_data.save();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn restart_journal_reader(&mut self) {
        let dir = PathBuf::from(&self.settings.journal_reader.journal_directory);
        let api_url = self.settings.api_url.trim().to_string();
        let url = if api_url.is_empty() { None } else { Some(api_url) };
        if dir.exists() {
            info!("Restarting journal reader with directory: {}", dir.display());
            if let Some(ref mut reader) = self.journal_reader {
                reader.restart(dir, url);
            } else {
                self.journal_reader = Some(JournalReader::start(dir, url));
            }
        } else {
            warn!("Journal directory does not exist: {}", dir.display());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn refresh_server_tick(&mut self) {
        let pending = std::sync::Arc::clone(&self.pending_tick);
        let api = self.api.clone();
        self.api.spawn(async move {
            if let Ok(Some(resp)) = api.get_server_tick().await {
                if let Some(tick) = resp.next_predicted_tick {
                    *pending.lock().unwrap() = Some(tick);
                }
            }
        });
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(frame.area());

        self.render_tabs(frame, chunks[0]);
        self.render_view(frame, chunks[1]);
        self.render_status_bar(frame, chunks[2]);
    }

    fn render_tabs(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let titles: Vec<Line> = TABS.iter().map(|t| Line::from(Span::raw(*t))).collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(APP_TITLE)
                    .style(Style::default().fg(Color::Rgb(255, 140, 0))),
            )
            .select(self.view.index())
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Rgb(255, 140, 0))
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::White))
            .divider(Span::raw("|"));

        frame.render_widget(tabs, area);
    }

    fn render_view(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.view {
            AppView::News => self.news.render(frame, area),
            AppView::Pilot => self.pilot.render(frame, area, &self.journal),
            AppView::Explorer => self.explorer.render(frame, area, &self.settings, &self.journal),
            AppView::Materials => self.inventory.render(frame, area, &self.journal),
            AppView::Modules => self.modules_view.render(frame, area, &self.journal),
            AppView::Stations => {
                let cs = self.carriers.my_carrier_stock(&self.journal);
                self.stations.render(frame, area, &self.journal, &self.todo_view.todo, &cs);
            }
            AppView::Carriers => {
                let cs = self.carriers.my_carrier_stock(&self.journal);
                self.carriers.render(frame, area, &self.journal, &self.todo_view.todo, &cs);
            }
            AppView::Factions => self.factions.render(frame, area),
            AppView::Construction => self.construction.render(frame, area, &self.journal, &self.todo_view.todo),
            AppView::TradeRoutes => self.trade_routes.render(frame, area, &self.journal),
            AppView::Engineers => self.engineers_view.render(frame, area),
            AppView::Todo => {
                let cs = self.carriers.my_carrier_stock(&self.journal);
                self.todo_view.render(frame, area, &self.journal, &cs);
            }
            AppView::SearchNearest => self.search_nearest.render(frame, area),
            AppView::Settings => self.settings_view.render(frame, area, &self.settings),
            AppView::About => self.about.render(frame, area),
        }
    }

    fn view_hints(&self) -> &'static [(&'static str, &'static str)] {
        match self.view {
            AppView::News         => &[],
            AppView::Pilot        => &[("w/s", "scroll")],
            AppView::Explorer     => &[("tab", "panel"), ("w/s", "factions"), ("a/d", "systems"), ("space", "open")],
            AppView::Materials    => &[("w/s", "navigate"), ("a/d", "panels")],
            AppView::Modules      => &[("w/s", "navigate")],
            AppView::Stations     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin")],
            AppView::Carriers     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("p", "pin"), ("m", "my carrier")],
            AppView::Factions     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin")],
            AppView::Construction => &[("f", "filter"), ("enter/tab", "panel"), ("w/s", "navigate"), ("t", "todo"), ("r", "remove")],
            AppView::TradeRoutes  => &[("tab", "cycle panels"), ("w/s", "navigate"), ("c", "copy system(s)"), ("r", "refresh")],
            AppView::Engineers    => &[("t", "ship/foot"), ("tab", "panel"), ("w/s", "navigate"), ("a/d", "grade"), ("enter", "add todo")],
            AppView::Todo         => &[("tab", "switch panel"), ("w/s", "navigate"), ("r/Del", "remove"), ("f", "search nearest")],
            AppView::SearchNearest => &[("enter", "edit/search"), ("esc", "stop editing"), ("tab", "switch field"), ("w/s", "navigate")],
            AppView::Settings     => &[("w/s", "rows"), ("a", "sidebar"), ("d", "fields"), ("space", "toggle"), ("enter", "save")],
            AppView::About        => &[],
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let bg         = Color::Rgb(255, 140, 0);
        let key_style  = Style::default().fg(Color::Black).bg(bg).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::Black).bg(bg);
        let sep_style  = Style::default().fg(Color::Rgb(120, 55, 0)).bg(bg);
        let info_style = Style::default().fg(Color::Rgb(60, 25, 0)).bg(bg);

        let mut left: Vec<Span<'static>> = Vec::new();

        let push_hint = |spans: &mut Vec<Span<'static>>, key: &'static str, desc: &'static str| {
            spans.push(Span::styled(format!(" {key}"), key_style));
            spans.push(Span::styled(format!(" {desc} "), desc_style));
        };

        push_hint(&mut left, "x", "quit");
        push_hint(&mut left, "q/e", "tabs");

        let hints = self.view_hints();
        if !hints.is_empty() {
            left.push(Span::styled(" │ ", sep_style));
            for (key, desc) in hints {
                push_hint(&mut left, key, desc);
            }
        }

        // Right-aligned section: system name + BGS tick countdown
        let system_name = self.journal.current_system.as_ref()
            .map(|s| s.name.clone())
            .unwrap_or_else(|| "—".to_string());

        let mut right: Vec<Span<'static>> = Vec::new();
        right.push(Span::styled("system ", info_style));
        right.push(Span::styled(system_name, desc_style));

        if let Some(next_tick) = self.next_tick {
            let now = chrono::Utc::now();
            let diff = next_tick.signed_duration_since(now);
            let tick_str = if diff.num_seconds() <= 0 {
                "tick: now!".to_owned()
            } else {
                let h = diff.num_hours();
                let m = diff.num_minutes() % 60;
                let s = diff.num_seconds() % 60;
                format!("tick in {:02}:{:02}:{:02}", h, m, s)
            };
            right.push(Span::styled(" │ ", sep_style));
            right.push(Span::styled(tick_str, desc_style));
        }
        right.push(Span::styled(" ", desc_style));

        // Fill the gap between left hints and right section so the bar spans the full width.
        let left_w:  usize = left.iter().map(|s| s.content.chars().count()).sum();
        let right_w: usize = right.iter().map(|s| s.content.chars().count()).sum();
        let sep_w = 3usize; // " │ "
        let pad = (area.width as usize).saturating_sub(left_w + sep_w + right_w);

        let mut spans = left;
        spans.push(Span::styled(" │ ", sep_style));
        spans.push(Span::styled(" ".repeat(pad), desc_style));
        spans.extend(right);

        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(bg)),
            area,
        );
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        let event = match self.view {
            AppView::News => self.news.handle_key(key),
            AppView::Pilot => self.pilot.handle_key(key),
            AppView::Explorer => self.explorer.handle_key(key),
            AppView::Materials => self.inventory.handle_key(key),
            AppView::Modules => self.modules_view.handle_key(key, &self.journal),
            AppView::Stations => self.stations.handle_key(key, &self.api, &self.journal),
            AppView::Carriers => self.carriers.handle_key(key, &self.api, &self.journal),
            AppView::Factions => self.factions.handle_key(key, &self.api),
            AppView::Construction => {
                let todo = &mut self.todo_view.todo;
                self.construction.handle_key(key, &self.api, &self.journal, todo)
            }
            AppView::TradeRoutes => self.trade_routes.handle_key(key, &self.api, &self.journal),
            AppView::Engineers => self.engineers_view.handle_key(key),
            AppView::Todo => self.todo_view.handle_key(key, &self.journal),
            AppView::SearchNearest => self.search_nearest.handle_key(key, &self.api),
            AppView::Settings => self.settings_view.handle_key(key, &mut self.settings),
            AppView::About => self.about.handle_key(key),
        };

        match event {
            ViewEvent::Consumed => return,
            ViewEvent::SettingsChanged => {
                info!("Settings changed, saving");
                self.settings_view.save_settings(&self.settings);
                #[cfg(not(target_arch = "wasm32"))]
                self.restart_journal_reader();
                return;
            }
            ViewEvent::OpenFactions(name) => {
                info!("Opening factions tab for: {}", name);
                self.factions.prefill_search(&name, &self.api);
                self.view = AppView::Factions;
                return;
            }
            ViewEvent::OpenSearchNearest { commodity, canonical_name, system, ship_pad_size } => {
                info!("Opening Search Nearest: commodity={}, canonical={}, system={}, pad={}", commodity, canonical_name, system, ship_pad_size);
                self.search_nearest.prefill_and_search(&commodity, &canonical_name, &system, ship_pad_size, &self.api);
                self.view = AppView::SearchNearest;
                return;
            }
            ViewEvent::None => {}
        }

        match key.code {
            KeyCode::Char('x') => {
                self.should_quit = true;
            }
            KeyCode::Char('e') => { self.go_next_tab(); }
            KeyCode::Char('q') => { self.go_prev_tab(); }
            _ => {}
        }
    }

    fn go_next_tab(&mut self) {
        self.view = self.view.next();
        info!("Tab changed to: {}", TABS[self.view.index()]);
        self.on_tab_enter();
    }

    fn go_prev_tab(&mut self) {
        self.view = self.view.prev();
        info!("Tab changed to: {}", TABS[self.view.index()]);
        self.on_tab_enter();
    }

    pub fn on_tab_enter(&mut self) {
        match self.view {
            AppView::News => self.news.start_fetch(&self.api),
            AppView::Stations => {
                let history = &self.journal.visited_stations;
                self.stations.on_enter(&self.api, history);
            }
            AppView::Carriers => self.carriers.on_enter(&self.api),
            AppView::Factions => self.factions.on_enter(&self.api),
            AppView::Construction => self.construction.on_enter(&self.api),
            AppView::TradeRoutes => {
                let journal = &self.journal;
                self.trade_routes.on_enter(&self.api, journal);
            }
            AppView::Todo => {}  // in-memory todo is always authoritative; no reload needed
            _ => {}
        }
    }

    pub fn poll_search_results(&mut self) {
        self.news.poll();
        self.factions.poll_search();
        self.stations.poll_search();
        #[cfg(not(target_arch = "wasm32"))]
        self.stations.poll_auto_fetch();
        self.carriers.poll_search();
        self.construction.poll_search();
        self.search_nearest.poll_search();
        #[cfg(target_arch = "wasm32")] {
            self.trade_routes.poll_search();
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(t) = self.pending_tick.lock().unwrap().take() {
            self.next_tick = Some(t);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn body_from_api(br: &edcas_common::api::BodyResponse) -> BodyScan {
    let rings = br
        .rings
        .iter()
        .map(|r| BodyRing {
            name: r.name.clone(),
            ring_class: r.ring_class.clone(),
            mass_mt: r.mass_mt,
            inner_rad: r.inner_rad,
            outer_rad: r.outer_rad,
        })
        .collect();

    let materials = br
        .materials
        .iter()
        .map(|m| BodyMaterial {
            name: m.name.clone(),
            percent: m.percent,
        })
        .collect();

    let parents = br
        .parents
        .iter()
        .map(|p| BodyParent {
            body_id: p.parent_id,
            parent_type: match p.parent_type.as_str() {
                "Star" => ParentType::Star,
                "Planet" => ParentType::Planet,
                "Ring" => ParentType::Ring,
                _ => ParentType::Null,
            },
        })
        .collect();

    BodyScan {
        body_id: br.id,
        body_name: br.name.clone(),
        planet_class: if br.is_star {
            String::new()
        } else {
            br.body_class.clone().unwrap_or_default()
        },
        landable: br.landable,
        scan_type: "API".into(),
        distance_from_arrival_ls: br.distance_from_arrival_ls.unwrap_or(0.0),
        radius: br.radius.unwrap_or(0.0),
        mass_em: br.mass_em.unwrap_or(0.0),
        surface_temperature: br.surface_temperature.unwrap_or(0.0),
        surface_gravity: br.surface_gravity.unwrap_or(0.0),
        tidal_lock: br.tidal_lock,
        volcanism: br.volcanism.clone().unwrap_or_default(),
        atmosphere: br.atmosphere.clone().unwrap_or_default(),
        terraform_state: br.terraform_state.clone().unwrap_or_default(),
        star_type: if br.is_star {
            br.body_class.clone().unwrap_or_default()
        } else {
            String::new()
        },
        parents,
        rings,
        materials,
        estimated_value: br.estimated_value.unwrap_or(0),
        composition: None,
    }
}
