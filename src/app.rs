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
use crate::todo::TodoList;
use crate::views::{
    AboutView, CarriersView, ConstructionView, EngineersView, ExplorerView, FactionsView,
    InventoryView, ModulesView, NewsView, PilotView, SettingsView, StationsView, SuitView,
    SystemView, TodoView, TradeRoutesView, ViewEvent,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::journal_reader::JournalReader;

pub const APP_TITLE: &str = "EDCAS - Elite Dangerous Commander Assistant System";

pub const TABS: &[&str] = &[
    "News",
    "System",
    "Explorer",
    "Inventory",
    "Stations",
    "Carriers",
    "Factions",
    "Construction",
    "Trade Routes",
    "Pilot",
    "Ship",
    "Suit",
    "Engineers",
    "Todo",
    "Settings",
    "About",
];

#[derive(Default, Clone, Copy, PartialEq)]
pub enum AppView {
    #[default]
    News = 0,
    System = 1,
    Explorer = 2,
    Materials = 3,
    Stations = 4,
    Carriers = 5,
    Factions = 6,
    Construction = 7,
    TradeRoutes = 8,
    Pilot = 9,
    Modules = 10,
    Suit = 11,
    Engineers = 12,
    Todo = 13,
    Settings = 14,
    About = 15,
}

impl AppView {
    pub fn next(&self) -> Self {
        match self {
            AppView::News => AppView::System,
            AppView::System => AppView::Explorer,
            AppView::Explorer => AppView::Materials,
            AppView::Materials => AppView::Stations,
            AppView::Stations => AppView::Carriers,
            AppView::Carriers => AppView::Factions,
            AppView::Factions => AppView::Construction,
            AppView::Construction => AppView::TradeRoutes,
            AppView::TradeRoutes => AppView::Pilot,
            AppView::Pilot => AppView::Modules,
            AppView::Modules => AppView::Suit,
            AppView::Suit => AppView::Engineers,
            AppView::Engineers => AppView::Todo,
            AppView::Todo => AppView::Settings,
            AppView::Settings => AppView::About,
            AppView::About => AppView::News,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            AppView::News => AppView::About,
            AppView::System => AppView::News,
            AppView::Explorer => AppView::System,
            AppView::Materials => AppView::Explorer,
            AppView::Stations => AppView::Materials,
            AppView::Carriers => AppView::Stations,
            AppView::Factions => AppView::Carriers,
            AppView::Construction => AppView::Factions,
            AppView::TradeRoutes => AppView::Construction,
            AppView::Pilot => AppView::TradeRoutes,
            AppView::Modules => AppView::Pilot,
            AppView::Suit => AppView::Modules,
            AppView::Engineers => AppView::Suit,
            AppView::Todo => AppView::Engineers,
            AppView::Settings => AppView::Todo,
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
    pub api: ApiClient,
    #[cfg(not(target_arch = "wasm32"))]
    pub api_rx: Option<std::sync::mpsc::Receiver<Vec<edcas_common::api::BodyResponse>>>,
    #[cfg(not(target_arch = "wasm32"))]
    pub last_api_system: i64,
    pub news: NewsView,
    pub pilot: PilotView,
    pub system: SystemView,
    pub explorer: ExplorerView,
    pub inventory: InventoryView,
    pub modules_view: ModulesView,
    pub suit_view: SuitView,
    pub stations: StationsView,
    pub carriers: CarriersView,
    pub factions: FactionsView,
    pub construction: ConstructionView,
    pub trade_routes: TradeRoutesView,
    pub engineers_view: EngineersView,
    pub todo_view: TodoView,
    pub settings_view: SettingsView,
    pub about: AboutView,
    pub should_quit: bool,
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

        let api = ApiClient::new(&settings.api_url);

        let mut app = Self {
            view: AppView::default(),
            settings,
            journal,
            journal_reader,
            api,
            api_rx: None,
            last_api_system: 0,
            news: NewsView::new(),
            pilot: PilotView::new(),
            system: SystemView::new(),
            explorer: ExplorerView::new(),
            inventory: InventoryView::new(),
            modules_view: ModulesView::new(),
            suit_view: SuitView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            factions: FactionsView::new(),
            construction: ConstructionView::new(),
            trade_routes: TradeRoutesView::new(),
            engineers_view: EngineersView::new(),
            todo_view: TodoView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
        };
        app.news.start_fetch(app.api.http_client());
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
            system: SystemView::new(),
            explorer: ExplorerView::new(),
            inventory: InventoryView::new(),
            modules_view: ModulesView::new(),
            suit_view: SuitView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            factions: FactionsView::new(),
            construction: ConstructionView::new(),
            trade_routes: TradeRoutesView::new(),
            engineers_view: EngineersView::new(),
            todo_view: TodoView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn poll_journal_updates(&mut self) {
        if let Some(ref reader) = self.journal_reader {
            while let Some(data) = reader.try_recv() {
                let system_name = data.current_system.as_ref().map(|s| s.name.clone()).unwrap_or_default();
                let body_count = data.bodies.len();
                info!("Journal update received: system={}, bodies={}", system_name, body_count);

                let new_addr = data.current_system.as_ref().map(|s| s.system_address).unwrap_or(0);
                if new_addr != 0 && new_addr != self.last_api_system && !self.settings.api_url.is_empty() {
                    self.last_api_system = new_addr;
                    let (tx, rx) = std::sync::mpsc::channel();
                    self.api_rx = Some(rx);
                    let base_url = self.settings.api_url.clone();
                    std::thread::spawn(move || {
                        let client = ApiClient::new(base_url);
                        if let Ok(bodies) = client.get_bodies(new_addr) {
                            let _ = tx.send(bodies);
                        }
                    });
                }

                self.journal = data;
                self.explorer.update(&self.journal);

                self.construction.update_from_journal(&self.api, &self.journal);
            }
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
                        self.journal.bodies.push(body_from_api(br));
                        added += 1;
                    }
                }
                if added > 0 {
                    info!("Merged {} bodies from API into explorer", added);
                    self.explorer.update(&self.journal);
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
            AppView::System => self.system.render(frame, area, &self.journal),
            AppView::Explorer => self.explorer.render(frame, area, &self.settings),
            AppView::Materials => self.inventory.render(frame, area, &self.journal),
            AppView::Modules => self.modules_view.render(frame, area, &self.journal),
            AppView::Suit    => self.suit_view.render(frame, area, &self.journal),
            AppView::Stations => self.stations.render(frame, area),
            AppView::Carriers => self.carriers.render(frame, area),
            AppView::Factions => self.factions.render(frame, area),
            AppView::Construction => self.construction.render(frame, area, &self.journal, &self.todo_view.todo),
            AppView::TradeRoutes => self.trade_routes.render(frame, area, &self.journal),
            AppView::Engineers => self.engineers_view.render(frame, area),
            AppView::Todo => self.todo_view.render(frame, area, &self.journal),
            AppView::Settings => self.settings_view.render(frame, area, &self.settings),
            AppView::About => self.about.render(frame, area),
        }
    }

    fn view_hints(&self) -> &'static [(&'static str, &'static str)] {
        match self.view {
            AppView::News         => &[],
            AppView::Pilot        => &[("w/s", "scroll")],
            AppView::System       => &[("w/s", "scroll")],
            AppView::Explorer     => &[("w/s", "navigate"), ("p", "pin")],
            AppView::Materials    => &[("w/s", "navigate"), ("a/d", "panels")],
            AppView::Modules      => &[("w/s", "navigate")],
            AppView::Suit         => &[("w/s", "navigate")],
            AppView::Stations     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin")],
            AppView::Carriers     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("p", "pin")],
            AppView::Factions     => &[("enter", "search"), ("w/s", "navigate"), ("tab", "panel"), ("a/d", "sub-tabs"), ("c", "copy system"), ("p", "pin")],
            AppView::Construction => &[("f", "filter"), ("enter/tab", "panel"), ("w/s", "navigate"), ("t", "todo"), ("r", "remove")],
            AppView::TradeRoutes  => &[("tab", "filter"), ("enter", "search"), ("w/s", "navigate"), ("r", "refresh")],
            AppView::Engineers    => &[("t", "ship/foot"), ("tab", "panel"), ("w/s", "navigate"), ("a/d", "grade"), ("enter", "add todo")],
            AppView::Todo         => &[("w/s", "navigate"), ("x", "remove")],
            AppView::Settings     => &[("w/s", "rows"), ("a", "sidebar"), ("d", "fields"), ("space", "toggle"), ("enter", "save")],
            AppView::About        => &[],
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let bg = Color::Rgb(255, 140, 0);
        let key_style  = Style::default().fg(Color::Black).bg(bg).add_modifier(Modifier::BOLD);
        let desc_style = Style::default().fg(Color::Black).bg(bg);
        let sep_style  = Style::default().fg(Color::Rgb(120, 55, 0)).bg(bg);
        let info_style = Style::default().fg(Color::Rgb(60, 25, 0)).bg(bg);

        let mut spans: Vec<Span<'static>> = Vec::new();

        let push_hint = |spans: &mut Vec<Span<'static>>, key: &'static str, desc: &'static str| {
            spans.push(Span::styled(format!(" {key}"), key_style));
            spans.push(Span::styled(format!(" {desc} "), desc_style));
        };

        push_hint(&mut spans, "x", "quit");
        push_hint(&mut spans, "q/e", "tabs");

        let hints = self.view_hints();
        if !hints.is_empty() {
            spans.push(Span::styled(" │ ", sep_style));
            for (key, desc) in hints {
                push_hint(&mut spans, key, desc);
            }
        }

        // Right-side system info
        let system_name = self.journal.current_system.as_ref().map(|s| s.name.as_str()).unwrap_or("—");
        spans.push(Span::styled(" │ ", sep_style));
        spans.push(Span::styled(format!("system "), info_style));
        spans.push(Span::styled(format!("{system_name}"), desc_style));

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }

    pub fn handle_key(&mut self, key: &KeyEvent) {
        let event = match self.view {
            AppView::News => self.news.handle_key(key),
            AppView::Pilot => self.pilot.handle_key(key),
            AppView::System => self.system.handle_key(key, &self.journal),
            AppView::Explorer => self.explorer.handle_key(key),
            AppView::Materials => self.inventory.handle_key(key),
            AppView::Modules => self.modules_view.handle_key(key, &self.journal),
            AppView::Suit    => self.suit_view.handle_key(key, &self.journal),
            AppView::Stations => self.stations.handle_key(key, &self.api),
            AppView::Carriers => self.carriers.handle_key(key, &self.api),
            AppView::Factions => self.factions.handle_key(key, &self.api),
            AppView::Construction => {
                let todo = &mut self.todo_view.todo;
                self.construction.handle_key(key, &self.api, &self.journal, todo)
            }
            AppView::TradeRoutes => self.trade_routes.handle_key(key, &self.api, &self.journal),
            AppView::Engineers => self.engineers_view.handle_key(key),
            AppView::Todo => self.todo_view.handle_key(key),
            AppView::Settings => self.settings_view.handle_key(key, &mut self.settings),
            AppView::About => self.about.handle_key(key),
        };

        match event {
            ViewEvent::Consumed => return,
            ViewEvent::NextTab => {
                self.view = self.view.next();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                self.on_tab_enter();
                return;
            }
            ViewEvent::PrevTab => {
                self.view = self.view.prev();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                self.on_tab_enter();
                return;
            }
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
            ViewEvent::None | ViewEvent::Quit => {}
        }

        match key.code {
            KeyCode::Char('x') => {
                self.should_quit = true;
            }
            KeyCode::Char('e') => {
                self.view = self.view.next();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                self.on_tab_enter();
            }
            KeyCode::Char('q') => {
                self.view = self.view.prev();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                self.on_tab_enter();
            }
            _ => {}
        }
    }

    pub fn on_tab_enter(&mut self) {
        match self.view {
            AppView::News => self.news.start_fetch(self.api.http_client()),
            AppView::Stations => self.stations.on_enter(&self.api),
            AppView::Carriers => self.carriers.on_enter(&self.api),
            AppView::Factions => self.factions.on_enter(&self.api),
            AppView::Construction => self.construction.on_enter(&self.api),
            AppView::TradeRoutes => {
                let journal = &self.journal;
                self.trade_routes.on_enter(&self.api, journal);
            }
            AppView::Todo => {
                self.todo_view.todo = TodoList::load();
            }
            _ => {}
        }
    }

    pub fn poll_search_results(&mut self) {
        self.news.poll();
        self.factions.poll_search();
        self.stations.poll_search();
        self.carriers.poll_search();
        self.construction.poll_search();
        #[cfg(target_arch = "wasm32")] {
            self.trade_routes.poll_search();
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
