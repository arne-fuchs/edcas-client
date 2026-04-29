extern crate core;

use std::io::stdout;
use std::process::exit;

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

use crate::desktop::settings::Settings;
use crate::views::{
    AboutView, CarriersView, ExplorerView, MaterialsView, MiningView, NewsView, SettingsView,
    StationsView, ViewEvent,
};

mod cli;
mod desktop;
mod edcas;
#[cfg(feature = "eddn")]
mod eddn;
mod views;

const APP_TITLE: &str = "EDCAS - Elite Dangerous Commander Assistant System";

const TABS: &[&str] = &[
    "News", "Explorer", "Mining", "Materials", "Stations", "Carriers", "Settings", "About",
];

#[derive(Default, Clone, Copy, PartialEq)]
enum AppView {
    #[default]
    News = 0,
    Explorer = 1,
    Mining = 2,
    Materials = 3,
    Stations = 4,
    Carriers = 5,
    Settings = 6,
    About = 7,
}

impl AppView {
    fn next(&self) -> Self {
        match self {
            AppView::News => AppView::Explorer,
            AppView::Explorer => AppView::Mining,
            AppView::Mining => AppView::Materials,
            AppView::Materials => AppView::Stations,
            AppView::Stations => AppView::Carriers,
            AppView::Carriers => AppView::Settings,
            AppView::Settings => AppView::About,
            AppView::About => AppView::News,
        }
    }

    fn prev(&self) -> Self {
        match self {
            AppView::News => AppView::About,
            AppView::Explorer => AppView::News,
            AppView::Mining => AppView::Explorer,
            AppView::Materials => AppView::Mining,
            AppView::Stations => AppView::Materials,
            AppView::Carriers => AppView::Stations,
            AppView::Settings => AppView::Carriers,
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
    news: NewsView,
    explorer: ExplorerView,
    mining: MiningView,
    materials: MaterialsView,
    stations: StationsView,
    carriers: CarriersView,
    settings_view: SettingsView,
    about: AboutView,
    should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            view: AppView::default(),
            settings: Settings::default(),
            news: NewsView::new(),
            explorer: ExplorerView::new(),
            mining: MiningView::new(),
            materials: MaterialsView::new(),
            stations: StationsView::new(),
            carriers: CarriersView::new(),
            settings_view: SettingsView::new(),
            about: AboutView::new(),
            should_quit: false,
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
        let titles: Vec<Line> = TABS
            .iter()
            .map(|t| Line::from(Span::raw(*t)))
            .collect();

        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(APP_TITLE)
                    .style(Style::default().fg(Color::Yellow)),
            )
            .select(self.view.index())
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .style(Style::default().fg(Color::White))
            .divider(Span::raw("|"));

        frame.render_widget(tabs, area);
    }

    fn render_view(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        match self.view {
            AppView::News => self.news.render(frame, area),
            AppView::Explorer => self.explorer.render(frame, area),
            AppView::Mining => self.mining.render(frame, area),
            AppView::Materials => self.materials.render(frame, area),
            AppView::Stations => self.stations.render(frame, area),
            AppView::Carriers => self.carriers.render(frame, area),
            AppView::Settings => self.settings_view.render(frame, area, &self.settings),
            AppView::About => self.about.render(frame, area),
        }
    }

    fn render_status_bar(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let editing_hint = if self.view == AppView::Settings {
            " | w/s: nav | a/d: section | space: edit | enter: save | esc: cancel | tab: switch tab"
        } else {
            ""
        };
        let status = format!(
            " x: quit | q/e: tabs{} | {}: {}",
            editing_hint,
            TABS[self.view.index()],
            match self.view {
                AppView::News => "Fetching Galnet articles...",
                _ => "",
            }
        );
        let status_bar = Paragraph::new(status).style(
            Style::default()
                .fg(Color::Yellow)
                .bg(Color::Black)
                .add_modifier(Modifier::REVERSED),
        );
        frame.render_widget(status_bar, area);
    }

    fn handle_key(&mut self, key: &crossterm::event::KeyEvent) {
        if key.code == crossterm::event::KeyCode::Char('x') {
            self.should_quit = true;
            return;
        }

        if key.code == crossterm::event::KeyCode::Char('e') {
            self.view = self.view.next();
            return;
        }
        if key.code == crossterm::event::KeyCode::Char('q') {
            self.view = self.view.prev();
            return;
        }

        let event = match self.view {
            AppView::News => self.news.handle_key(key),
            AppView::Explorer => self.explorer.handle_key(key),
            AppView::Mining => self.mining.handle_key(key),
            AppView::Materials => self.materials.handle_key(key),
            AppView::Stations => self.stations.handle_key(key),
            AppView::Carriers => self.carriers.handle_key(key),
            AppView::Settings => self.settings_view.handle_key(key, &mut self.settings),
            AppView::About => self.about.handle_key(key),
        };

        match event {
            ViewEvent::NextTab => self.view = self.view.next(),
            ViewEvent::PrevTab => self.view = self.view.prev(),
            ViewEvent::SettingsChanged => self.settings_view.save_settings(&self.settings),
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let ascii_art = r#"
  ______    ____      ____       _       ______
 |  ____|  |  _ \   / ____|     / \     /  ____|
 | |__     | | | |  | |        / _ \    |  (___
 |  __|    | | | |  | |       / /_\ \   \___   \
 | |____   | |_| |  | |____  /  / \  \   ____) |
 |______|  |____/   \_____| /__/   \__\ |_____/

"#;
    println!("{}", ascii_art);

    for arg in args {
        match arg.as_str() {
            #[cfg(feature = "eddn")]
            "--eddn-listener" => {
                eddn::run_listener();
                exit(0);
            }
            #[cfg(feature = "eddn")]
            "--eddn-parser" => {
                eddn::run_parser();
                exit(0);
            }
            "--help" => {
                println!("--eddn-parser Runs the parser on the database");
                println!("--eddn-listener Listens to the eddn network and loads the json into the database");
                exit(0);
            }
            _ => {}
        }
    }

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    enable_raw_mode()?;
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
        eprintln!("Error: {:?}", err);
        exit(1);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                app.handle_key(&key);
                if app.should_quit {
                    return Ok(());
                }
            }
        }
    }
}
