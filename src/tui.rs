use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::Key, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::app::EliteRustClient;

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub tab_index: usize,
    pub body_index: usize,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Explorer", "Mining", "Materials", "About"],
            tab_index: 0,
            body_index: 0,
        }
    }

    pub fn next_tab(&mut self) {
        self.tab_index = (self.tab_index + 1) % self.titles.len();
    }

    pub fn previous_tab(&mut self) {
        if self.tab_index > 0 {
            self.tab_index -= 1;
        } else {
            self.tab_index = self.titles.len() - 1;
        }
    }
    pub fn next_system(&mut self, client: &mut EliteRustClient) {
        if client.explorer.index + 1 < client.explorer.systems.len() {
            client.explorer.index += 1;
        }
    }

    pub fn previous_system(&mut self, client: &mut EliteRustClient) {
        if client.explorer.index - 1 > 0 {
            client.explorer.index -= 1;
        }
    }

    // TODO: add functions for cursor navigation through signals/bodies lists
}

pub fn draw_tui(client: EliteRustClient) -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app, client);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        //DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    mut client: EliteRustClient,
) -> io::Result<()> {
    loop {
        client.update_values();

        terminal.draw(|f| ui(f, &app, &client))?;

        if event::poll(std::time::Duration::from_millis(33))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Char('e') => app.next_tab(),
                        KeyCode::Char('q') => app.previous_tab(),
                        KeyCode::Right => app.next_system(&mut client),
                        KeyCode::Left => app.previous_system(&mut client),
                        // TODO: add keys for cursor navigation through signals/bodies list
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App, client: &EliteRustClient) {
    let size = f.size();
    //definition of general layout
    let chunks = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(size);

    let tabs_and_timestamp = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(22)])
        .split(chunks[0]);

    let titles: Vec<&str> = app.titles.clone();
    //render tabs
    let tabs = Tabs::new(titles)
        .block(
            Block::default().borders(Borders::NONE).white(), //.on_black(),
        )
        .select(app.tab_index)
        .style(Style::default().white())
        .highlight_style(Style::default().bold().white().on_gray());
    f.render_widget(tabs, tabs_and_timestamp[0]);

    let timestamp = Paragraph::new(client.timestamp.clone())
        .white()
        .block(Block::default().borders(Borders::LEFT).white());
    f.render_widget(timestamp, tabs_and_timestamp[1]);

    //render tab contents
    match app.tab_index {
        0 => tab_explorer(chunks[1], f, client, app),
        1 => tab_mining(chunks[1], f, client, app),
        2 => tab_materials(chunks[1], f, client),
        3 => tab_about(chunks[1], f),
        _ => unreachable!(),
    };
}

// ======== Tabs functions =========

fn tab_default(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame, _client: &EliteRustClient) {
    let widget_default = Paragraph::new("default text here").block(
        Block::default()
            .title("Default")
            .borders(Borders::ALL)
            .white(),
    );

    f.render_widget(widget_default, chunk);
}

fn tab_explorer(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &App,
) {
    //general layout
    let layout_explorer = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Fill(1),
            Constraint::Percentage(25),
        ])
        .split(chunk);

    let layout_system = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(10), Constraint::Fill(1)])
        .split(layout_explorer[0]);

    let mut data_system_info = vec!["no data".to_string()];
    let mut data_signals_list = vec!["no data".to_string()];
    let mut data_body_list = vec!["no data".to_string()];
    let mut data_body_signals_list = vec!["no data".to_string()];

    if !client.explorer.systems.is_empty() {
        data_system_info = vec![
            client.explorer.systems[client.explorer.index].name.clone(),
            client.explorer.systems[client.explorer.index]
                .allegiance
                .clone(),
            client.explorer.systems[client.explorer.index]
                .economy_localised
                .clone(),
            client.explorer.systems[client.explorer.index]
                .second_economy_localised
                .clone(),
            client.explorer.systems[client.explorer.index]
                .government_localised
                .clone(),
            client.explorer.systems[client.explorer.index]
                .security_localised
                .clone(),
            client.explorer.systems[client.explorer.index]
                .population
                .clone(),
            client.explorer.systems[client.explorer.index]
                .body_count
                .clone(),
            client.explorer.systems[client.explorer.index]
                .non_body_count
                .clone(),
        ];
        data_signals_list = client.explorer.systems[client.explorer.index]
            .signal_list
            .iter()
            .map(|signal| signal.clone().name)
            .collect::<Vec<_>>();

        if !client.explorer.systems[client.explorer.index]
            .body_list
            .is_empty()
        {
            data_body_list = client.explorer.systems[client.explorer.index]
                .body_list
                .iter()
                .map(|body| {
                    let mut space_string = "".to_string();
                    for i in 0..body.get_parents().len() {
                        if i < body.get_parents().len() - 1 {
                            space_string.push_str("| ")
                        } else {
                            space_string.push_str("|-");
                        }
                    }
                    space_string.push_str(body.get_name());
                    space_string
                })
                .collect::<Vec<_>>();

            if !client.explorer.systems[client.explorer.index].body_list
                [client.explorer.systems[client.explorer.index].index]
                .get_signals()
                .is_empty()
            {
                data_body_signals_list = client.explorer.systems[client.explorer.index].body_list
                    [client.explorer.systems[client.explorer.index].index]
                    .get_signals()
                    .iter()
                    .map(|body_signal| {
                        if body_signal.type_localised != "null" {
                            body_signal.type_localised.clone()
                        } else {
                            body_signal.r#type.clone()
                        }
                    })
                    .collect::<Vec<_>>();
            }
        }
    }

    let widget_system_info = List::new(data_system_info).block(
        Block::default()
            .title("System Info")
            .borders(Borders::ALL)
            .white(),
    );
    f.render_widget(widget_system_info, layout_system[0]);

    //widget for signals list
    let widget_signal_list = List::new(data_signals_list).block(
        Block::default()
            .title("Signals")
            .borders(Borders::ALL)
            .white(),
    );

    f.render_widget(widget_signal_list, layout_system[1]);

    //widget for bodies list
    // TODO: automatical generation of sub-layouts for body features like rings or planets
    let widget_body_list = List::new(data_body_list)
        .block(
            Block::default()
                .title("Body List")
                .borders(Borders::ALL)
                .white(),
        )
        .highlight_style(Style::default().white().on_gray());

    f.render_widget(widget_body_list, layout_explorer[1]);

    let widget_body_signals_list = List::new(data_body_signals_list).block(
        Block::default()
            .title("Body Signals")
            .borders(Borders::ALL)
            .white(),
    );

    f.render_widget(widget_body_signals_list, layout_explorer[2]);
}

fn tab_mining(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &App,
) {
    //TODO:
}

fn tab_materials(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame, client: &EliteRustClient) {
    //TODO:
}

fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame) {
    // ob ich den layout überhaupt brauhe?
    let layout_about = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(chunk);

    let widget_about_github = Paragraph::new("https://github.com/arne-fuchs/edcas-client").block(
        Block::default()
            .title("GitHub")
            .borders(Borders::ALL)
            .white(),
    );
    f.render_widget(widget_about_github, layout_about[1]);

    let widget_about_version = Paragraph::new(env!("CARGO_PKG_VERSION")).block(
        Block::default()
            .title("edcas version")
            .borders(Borders::ALL)
            .white(),
    );
    f.render_widget(widget_about_version, layout_about[2]);

    let widget_about_controls =
        Paragraph::new("Quit: Q, Change Tabs: q and e, Change System: Left and Right arrows")
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::ALL)
                    .white(),
            );
    f.render_widget(widget_about_controls, layout_about[0]);
}
