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
    CommanderView, ExplorerView, FactionsView,
    NewsView, SearchNearestView, SettingsView, StationsView,
    TodoView, TradeRoutesView, ViewEvent, WorkshopView,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::journal_reader::JournalReader;

pub const APP_TITLE: &str = "EDCAS - Elite Dangerous Commander Assistant System";

pub const TABS: &[&str] = &[
    "News",
    "Explorer",
    "Stations",
    "Factions",
    "Trade",
    "Commander",
    "Workshop",
    "Todo",
    "Settings",
];

#[derive(Default, Clone, Copy, PartialEq)]
pub enum AppView {
    #[default]
    News = 0,
    Explorer = 1,
    Stations = 2,
    Factions = 3,
    TradeRoutes = 4,
    Commander = 5,
    Workshop = 6,
    Todo = 7,
    Settings = 8,
    /// Not in the tab bar — reached only via context (e.g. `f` in Todo).
    SearchNearest = 9,
}

impl AppView {
    pub fn next(&self) -> Self {
        match self {
            AppView::News => AppView::Explorer,
            AppView::Explorer => AppView::Stations,
            AppView::Stations => AppView::Factions,
            AppView::Factions => AppView::TradeRoutes,
            AppView::TradeRoutes => AppView::Commander,
            AppView::Commander => AppView::Workshop,
            AppView::Workshop => AppView::Todo,
            AppView::Todo => AppView::Settings,
            AppView::Settings => AppView::News,
            AppView::SearchNearest => AppView::Settings,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            AppView::News => AppView::Settings,
            AppView::Explorer => AppView::News,
            AppView::Stations => AppView::Explorer,
            AppView::Factions => AppView::Stations,
            AppView::TradeRoutes => AppView::Factions,
            AppView::Commander => AppView::TradeRoutes,
            AppView::Workshop => AppView::Commander,
            AppView::Todo => AppView::Workshop,
            AppView::SearchNearest => AppView::Todo,
            AppView::Settings => AppView::Todo,
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
    pub commander: CommanderView,
    pub explorer: ExplorerView,
    pub workshop: WorkshopView,
    pub stations: StationsView,
    pub factions: FactionsView,
    pub trade_routes: TradeRoutesView,
    pub todo_view: TodoView,
    pub search_nearest: SearchNearestView,
    pub settings_view: SettingsView,
    pub should_quit: bool,
    pub next_tick: Option<chrono::DateTime<chrono::Utc>>,
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
            commander: CommanderView::new(),
            explorer: ExplorerView::new(),
            workshop: WorkshopView::new(),
            stations: StationsView::new(),
            factions: FactionsView::new(),
            trade_routes: TradeRoutesView::new(),
            todo_view: TodoView::new(),
            search_nearest: SearchNearestView::new(),
            settings_view: SettingsView::new(),
            should_quit: false,
            next_tick: None,
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
            commander: CommanderView::new(),
            explorer: ExplorerView::new(),
            workshop: WorkshopView::new(),
            stations: StationsView::new(),
            factions: FactionsView::new(),
            trade_routes: TradeRoutesView::new(),
            todo_view: TodoView::new(),
            search_nearest: SearchNearestView::new(),
            settings_view: SettingsView::new(),
            should_quit: false,
            next_tick: None,
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

            // Keep the stations view selection on the same station even when
            // visited_stations changes order (e.g., a new dock prepends a station).
            self.stations.on_journal_update(&self.journal);

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

            self.stations.update_construction_from_journal(&self.api, &self.journal, &mut self.todo_view.todo);
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
            AppView::Commander => self.commander.render(frame, area, &self.journal),
            AppView::Explorer => self.explorer.render(frame, area, &self.settings, &self.journal),
            AppView::Workshop => self.workshop.render(frame, area, &self.journal),
            AppView::Stations => {
                self.stations.render(frame, area, &self.journal, &self.todo_view.todo);
            }
            AppView::Factions => self.factions.render(frame, area),
            AppView::TradeRoutes => self.trade_routes.render(frame, area, &self.journal),
            AppView::Todo => {
                self.todo_view.render(frame, area, &self.journal);
            }
            AppView::SearchNearest => self.search_nearest.render(frame, area),
            AppView::Settings => self.settings_view.render(frame, area, &self.settings),
        }
    }

    fn view_hints(&self) -> &'static [(&'static str, &'static str)] {
        match self.view {
            AppView::News        => &[],
            AppView::Commander   => &[("tab", "switch view"), ("w/s", "scroll/navigate")],
            AppView::Explorer    => &[("tab", "panel"), ("w/s", "factions"), ("a/d", "systems"), ("space", "open")],
            AppView::Workshop    => &[("1/2", "switch view"), ("w/s", "navigate"), ("a/d", "panels/grade")],
            AppView::Stations    => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin"), ("t", "todo"), ("f", "nearest")],
            AppView::Factions    => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin")],
            AppView::TradeRoutes => &[("tab", "cycle panels"), ("w/s", "navigate"), ("c", "copy system(s)"), ("r", "refresh")],
            AppView::Todo        => &[("tab", "switch panel"), ("w/s", "navigate"), ("r/Del", "remove"), ("f", "search nearest")],
            AppView::SearchNearest => &[("w/s", "navigate"), ("q", "back to Todo")],
            AppView::Settings    => &[("w/s", "rows"), ("a", "sidebar"), ("d", "fields"), ("space", "toggle"), ("enter", "save")],
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
            AppView::Commander => self.commander.handle_key(key, &self.journal),
            AppView::Explorer => self.explorer.handle_key(key),
            AppView::Workshop => self.workshop.handle_key(key),
            AppView::Stations => self.stations.handle_key(key, &self.api, &self.journal, &mut self.todo_view.todo),
            AppView::Factions => self.factions.handle_key(key, &self.api),
            AppView::TradeRoutes => self.trade_routes.handle_key(key, &self.api, &self.journal),
            AppView::Todo => self.todo_view.handle_key(key, &self.journal),
            AppView::SearchNearest => self.search_nearest.handle_key(key, &self.api),
            AppView::Settings => self.settings_view.handle_key(key, &mut self.settings),
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
            ViewEvent::TrackConstruction(market_id) => {
                info!("Tracking construction site {} from Stations pin", market_id);
                self.stations.track_construction(market_id, &self.api);
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
        info!("Tab changed to: {}", TABS.get(self.view.index()).copied().unwrap_or("SearchNearest"));
        self.on_tab_enter();
    }

    fn go_prev_tab(&mut self) {
        self.view = self.view.prev();
        info!("Tab changed to: {}", TABS.get(self.view.index()).copied().unwrap_or("SearchNearest"));
        self.on_tab_enter();
    }

    pub fn on_tab_enter(&mut self) {
        match self.view {
            AppView::News => self.news.start_fetch(&self.api),
            AppView::Stations => {
                self.stations.on_enter(&self.api, &self.journal);
            }
            AppView::Factions => self.factions.on_enter(&self.api),
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
        self.stations.poll_construction();
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
