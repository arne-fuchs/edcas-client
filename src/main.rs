extern crate core;

use std::io::stdout;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame, Terminal,
};
use tracing::{info, warn, error};
use tracing_appender::non_blocking::WorkerGuard;

use crate::api_client::ApiClient;
use crate::settings::Settings;
use crate::journal_reader::{
    BodyMaterial, BodyParent, BodyRing, BodyScan, JournalData, JournalReader, ParentType,
};
use crate::views::{
    AboutView, CarriersView, ExplorerView, FactionsView, InventoryView, LogView, MiningView,
    NewsView, SettingsView, StationsView, SystemView, ViewEvent,
};

mod api_client;
mod cli;
mod journal_reader;
mod settings;
mod views;

const APP_TITLE: &str = "EDCAS - Elite Dangerous Commander Assistant System";

const TABS: &[&str] = &[
    "News",
    "System",
    "Explorer",
    "Mining",
    "Inventory",
    "Stations",
    "Carriers",
    "Factions",
    "Log",
    "Settings",
    "About",
];

#[derive(Default, Clone, Copy, PartialEq)]
enum AppView {
    #[default]
    News = 0,
    System = 1,
    Explorer = 2,
    Mining = 3,
    Materials = 4,
    Stations = 5,
    Carriers = 6,
    Factions = 7,
    Log = 8,
    Settings = 9,
    About = 10,
}

impl AppView {
    fn next(&self) -> Self {
        match self {
            AppView::News => AppView::System,
            AppView::System => AppView::Explorer,
            AppView::Explorer => AppView::Mining,
            AppView::Mining => AppView::Materials,
            AppView::Materials => AppView::Stations,
            AppView::Stations => AppView::Carriers,
            AppView::Carriers => AppView::Factions,
            AppView::Factions => AppView::Log,
            AppView::Log => AppView::Settings,
            AppView::Settings => AppView::About,
            AppView::About => AppView::News,
        }
    }

    fn prev(&self) -> Self {
        match self {
            AppView::News => AppView::About,
            AppView::System => AppView::News,
            AppView::Explorer => AppView::System,
            AppView::Mining => AppView::Explorer,
            AppView::Materials => AppView::Mining,
            AppView::Stations => AppView::Materials,
            AppView::Carriers => AppView::Stations,
            AppView::Factions => AppView::Carriers,
            AppView::Log => AppView::Factions,
            AppView::Settings => AppView::Log,
            AppView::About => AppView::Settings,
        }
    }

    fn index(&self) -> usize {
        *self as usize
    }
}

struct App {
    view: AppView,
    settings: Settings,
    journal: JournalData,
    journal_reader: Option<JournalReader>,
    api: ApiClient,
    api_rx: Option<std::sync::mpsc::Receiver<Vec<edcas_common::api::BodyResponse>>>,
    last_api_system: i64,
    news: NewsView,
    system: SystemView,
    explorer: ExplorerView,
    mining: MiningView,
    inventory: InventoryView,
    stations: StationsView,
    carriers: CarriersView,
    factions: FactionsView,
    log_view: LogView,
    settings_view: SettingsView,
    about: AboutView,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
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

        let journal_reader = journal_dir.map(|dir| {
            info!("Starting journal reader for directory: {}", dir.display());
            JournalReader::start(dir)
        });

        let api = ApiClient::new(&settings.api_url);

        Self {
            view: AppView::default(),
            settings,
            journal,
            journal_reader,
            api,
            api_rx: None,
            last_api_system: 0,
            news: NewsView::new(),
            system: SystemView::new(),
            explorer: ExplorerView::new(),
            mining: MiningView::new(),
            inventory: InventoryView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            factions: FactionsView::new(),
            log_view: LogView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
        }
    }

    fn poll_journal_updates(&mut self) {
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
            }
        }

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

    fn restart_journal_reader(&mut self) {
        let dir = PathBuf::from(&self.settings.journal_reader.journal_directory);
        if dir.exists() {
            info!("Restarting journal reader with directory: {}", dir.display());
            if let Some(ref mut reader) = self.journal_reader {
                reader.restart(dir);
            } else {
                self.journal_reader = Some(JournalReader::start(dir));
            }
        } else {
            warn!("Journal directory does not exist: {}", dir.display());
        }
    }

    fn render(&mut self, frame: &mut Frame) {
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
            AppView::System => self.system.render(frame, area, &self.journal),
            AppView::Explorer => self.explorer.render(frame, area),
            AppView::Mining => self.mining.render(frame, area, &self.journal),
            AppView::Materials => self.inventory.render(frame, area, &self.journal),
            AppView::Stations => self.stations.render(frame, area),
            AppView::Carriers => self.carriers.render(frame, area),
            AppView::Factions => self.factions.render(frame, area),
            AppView::Log => self.log_view.render(frame, area, &self.settings.journal_reader.journal_directory),
            AppView::Settings => self.settings_view.render(frame, area, &self.settings),
            AppView::About => self.about.render(frame, area),
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let journal_info = format!(
            "bodies: {} | ",
            self.journal.bodies.len()
        );
        let system_info = self.journal.current_system
            .as_ref()
            .map(|s| format!("system: {} | ", s.name))
            .unwrap_or_default();

        let editing_hint = if self.view == AppView::Settings {
            " | w/s: rows | a: sidebar | d: fields | space: select/edit | enter: save"
        } else {
            ""
        };
        let status = format!(
            " x: quit | q/e: tabs{} | {}{}{}",
            editing_hint,
            system_info,
            journal_info,
            TABS[self.view.index()],
        );
        let status_bar = Paragraph::new(status).style(
            Style::default()
                .fg(Color::Rgb(255, 140, 0))
                .bg(Color::Black)
                .add_modifier(Modifier::REVERSED),
        );
        frame.render_widget(status_bar, area);
    }

    fn handle_key(&mut self, key: &crossterm::event::KeyEvent) {
        let event = match self.view {
            AppView::News => self.news.handle_key(key),
            AppView::System => self.system.handle_key(key, &self.journal),
            AppView::Explorer => self.explorer.handle_key(key),
            AppView::Mining => self.mining.handle_key(key),
            AppView::Materials => self.inventory.handle_key(key),
            AppView::Stations => self.stations.handle_key(key, &self.api),
            AppView::Carriers => self.carriers.handle_key(key, &self.api),
            AppView::Factions => self.factions.handle_key(key, &self.api),
            AppView::Log => self.log_view.handle_key(key),
            AppView::Settings => self.settings_view.handle_key(key, &mut self.settings),
            AppView::About => self.about.handle_key(key),
        };

        match event {
            ViewEvent::Consumed => return,
            ViewEvent::NextTab => {
                self.view = self.view.next();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                return;
            }
            ViewEvent::PrevTab => {
                self.view = self.view.prev();
                info!("Tab changed to: {}", TABS[self.view.index()]);
                return;
            }
            ViewEvent::SettingsChanged => {
                info!("Settings changed, saving and restarting journal reader");
                self.settings_view.save_settings(&self.settings);
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

        // Global shortcuts — only reached when the active view did not consume the key
        match key.code {
            crossterm::event::KeyCode::Char('x') => {
                self.should_quit = true;
            }
            crossterm::event::KeyCode::Char('e') => {
                self.view = self.view.next();
                info!("Tab changed to: {}", TABS[self.view.index()]);
            }
            crossterm::event::KeyCode::Char('q') => {
                self.view = self.view.prev();
                info!("Tab changed to: {}", TABS[self.view.index()]);
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let _file_guard = init_file_logging();

    info!("EDCAS client starting");

    let args: Vec<String> = std::env::args().collect();
    info!("CLI arguments: {:?}", args);

    for arg in args {
        match arg.as_str() {
            "--help" => {
                println!("EDCAS client — Elite Dangerous Commander Assistant System");
                println!("Run `edcas-server` (separate binary) to start the EDDN listener and API.");
                exit(0);
            }
            _ => {}
        }
    }

    enable_raw_mode()?;
    info!("Terminal raw mode enabled");
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("Application error: {:?}", err);
        eprintln!("Error: {:?}", err);
        exit(1);
    }

    info!("Application exited cleanly");
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    info!("Entering main application loop");
    loop {
        app.poll_journal_updates();

        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    info!("Key pressed: {:?}", key);
                    app.handle_key(&key);
                    if app.should_quit {
                        info!("Quit requested");
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn body_from_api(br: &edcas_common::api::BodyResponse) -> BodyScan {
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

fn init_file_logging() -> WorkerGuard {
    let file = std::fs::File::create("log.log").expect("Failed to create log file");
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    guard
}
