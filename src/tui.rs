use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event::Key, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::app::{self, EliteRustClient};

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub tab_index: usize,
    pub body_list_state: ListState,
    pub material_index: usize,
    pub material_list_state: ListState,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Explorer", "Mining", "Materials", "About"],
            tab_index: 0,
            body_list_state: ListState::default(),
            material_index: 0,
            material_list_state: ListState::default(),
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
        if !client.explorer.systems.is_empty()
            && client.explorer.index + 1 < client.explorer.systems.len()
        {
            client.explorer.index += 1;
        }
    }

    pub fn previous_system(&mut self, client: &mut EliteRustClient) {
        if !client.explorer.systems.is_empty() && client.explorer.index > 0 {
            client.explorer.index -= 1;
        }
    }

    pub fn next_body(&mut self, client: &mut EliteRustClient) {
        if !client.explorer.systems.is_empty()
            && client.explorer.systems[client.explorer.index].index + 1
                < client.explorer.systems[client.explorer.index]
                    .body_list
                    .len()
        // TODO: fix: crashes the tui if the systems list is empty
        {
            client.explorer.systems[client.explorer.index].index += 1;
        }
    }

    pub fn previous_body(&mut self, client: &mut EliteRustClient) {
        if !client.explorer.systems.is_empty()
            && client.explorer.systems[client.explorer.index].index > 0
        {
            client.explorer.systems[client.explorer.index].index -= 1;
        }
    }

    pub fn next_material(&mut self, client: &mut EliteRustClient) {
        // TODO:
    }

    pub fn previous_material(&mut self, client: &mut EliteRustClient) {
        // TODO:
    }

    // TODO: add functions for cursor navigation through signals lists
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

        terminal.draw(|f| ui(f, &mut app, &client))?;

        if event::poll(std::time::Duration::from_millis(33))? {
            if let Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('Q') => return Ok(()),
                        KeyCode::Char('e') => app.next_tab(),
                        KeyCode::Char('q') => app.previous_tab(),
                        KeyCode::Right => app.next_system(&mut client),
                        KeyCode::Left => app.previous_system(&mut client),
                        KeyCode::Down => match app.tab_index {
                            0 => app.next_body(&mut client),
                            2 => app.next_material(&mut client),
                            _ => {}
                        },
                        KeyCode::Up => match app.tab_index {
                            0 => app.previous_body(&mut client),
                            2 => app.previous_material(&mut client),
                            _ => {}
                        },
                        // TODO: add keys for cursor navigation through signals list
                        _ => {}
                    }
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &mut App, client: &EliteRustClient) {
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
        .highlight_style(Style::default().bold().white().on_dark_gray());
    f.render_widget(tabs, tabs_and_timestamp[0]);

    let timestamp = Paragraph::new(client.timestamp.clone())
        .white()
        .block(Block::default().borders(Borders::LEFT).white());
    f.render_widget(timestamp, tabs_and_timestamp[1]);

    //render tab contents
    match app.tab_index {
        0 => tab_explorer(chunks[1], f, client, app),
        1 => tab_mining(chunks[1], f, client, app),
        2 => tab_materials(chunks[1], f, client, app),
        3 => tab_about(chunks[1], f),
        _ => unreachable!(),
    };
}

// ======== Tabs functions =========

// Explorer --------------------------------------------------------------------------------------------------------------------------------------------------------------------
fn tab_explorer(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    // Layout definitions
    // general layout
    let layout_explorer = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage(25),
            Constraint::Fill(1),
            Constraint::Percentage(25),
        ])
        .split(chunk);

    // layout of "systems" Panel
    let layout_system = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(11), Constraint::Fill(1)])
        .split(layout_explorer[0]);

    // layout of "body information" panel
    let layout_body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(20), Constraint::Fill(1)])
        .split(layout_explorer[2]);
    // Data
    // Default data to display
    let mut data_system_info = vec!["no data".to_string()];
    let mut data_signals_list = vec!["no data".to_string()];
    let mut data_body_list = vec!["no data".to_string()];
    let mut data_body_signals_list = vec!["no data".to_string()];
    let mut data_body_info = vec!["no data".to_string()];

    // Checks to not crash everything if list is empty
    // Data acquisition
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
                            space_string.push_str("│  ")
                        } else {
                            space_string.push_str("│  "); //├─
                        }
                    }
                    space_string.push_str(body.get_name());
                    space_string
                })
                .collect::<Vec<_>>();

            //TODO: parse json to Vec and use it here
            data_body_info = vec![client.explorer.systems[client.explorer.index].body_list
                [client.explorer.systems[client.explorer.index].index]
                .get_name()
                .to_string()];

            // Selection from body_list (cursor and scrolling)
            app.body_list_state
                .select(Some(client.explorer.systems[client.explorer.index].index));

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
            } else {
                data_body_signals_list = vec!["no signals".to_string()]
            }
        }
    }

    // Widget definitions
    let widget_system_info = List::new(data_system_info).block(
        Block::default()
            .title("System Info")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    let widget_signal_list = List::new(data_signals_list).block(
        Block::default()
            .title("Signals")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    let widget_body_list = List::new(data_body_list)
        .block(
            Block::default()
                .title("Body List")
                .borders(Borders::LEFT | Borders::TOP | Borders::RIGHT)
                .bold()
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_body_info = List::new(data_body_info).block(
        Block::default()
            .title("Body Info")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    let widget_body_signals_list = List::new(data_body_signals_list).block(
        Block::default()
            .title("Body Signals")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    // render calls
    f.render_widget(widget_system_info, layout_system[0]);
    f.render_widget(widget_signal_list, layout_system[1]);

    f.render_stateful_widget(
        widget_body_list,
        layout_explorer[1],
        &mut app.body_list_state,
    );

    f.render_widget(widget_body_info, layout_body[0]);
    f.render_widget(widget_body_signals_list, layout_body[1]);
}

// Mining --------------------------------------------------------------------------------------------------------------------------------------------------------------------

fn tab_mining(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    //TODO:
}

// Materials --------------------------------------------------------------------------------------------------------------------------------------------------------------------

fn tab_materials(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    //TODO:

    // Layout definitions
    let layout_materials = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(45), Constraint::Fill(1)])
        .split(chunk);

    let layout_materials_list = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Fill(1)])
        .split(layout_materials[0]);

    let layout_materials_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1), // Description
            Constraint::Fill(1), // Location
            Constraint::Fill(1), // Sources
            Constraint::Fill(1), // Engineering
            Constraint::Fill(1), // Synthesis
        ])
        .split(layout_materials[1]);

    // TODO: data acquisition

    // Selection from materials list (cursor and scrolling)
    app.material_list_state.select(Some(app.material_index));

    // Widget definitions
    // material_list field
    let widget_materials_list_names = List::new(["material 1", "material 2", "material 3"])
        .block(
            Block::default()
                .title("Materials")
                .borders(Borders::TOP)
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_materials_list_count = List::new(["100/200", "10/250", "0/100"])
        .block(Block::default().borders(Borders::TOP | Borders::RIGHT));

    // materials_info
    let widget_materials_info_description =
        Paragraph::new("description depending on selected material")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Description")
                    .bold()
                    .borders(Borders::TOP),
            );
    let widget_materials_info_location =
        Paragraph::new("location info depending on selected material")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Location")
                    .bold()
                    .borders(Borders::TOP),
            );
    let widget_materials_info_source =
        Paragraph::new("sources info depending on selected material")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Sources")
                    .bold()
                    .borders(Borders::TOP),
            );
    let widget_materials_info_engineering =
        Paragraph::new("engineering info depending on selected material")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Engineering")
                    .bold()
                    .borders(Borders::TOP),
            );
    let widget_materials_info_synthesis =
        Paragraph::new("synthesis info depending on selected material")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Synthesis")
                    .bold()
                    .borders(Borders::TOP),
            );

    // Render calls
    f.render_stateful_widget(
        widget_materials_list_names,
        layout_materials_list[0],
        &mut app.material_list_state,
    );
    f.render_widget(widget_materials_list_count, layout_materials_list[1]);

    f.render_widget(widget_materials_info_description, layout_materials_info[0]);
    f.render_widget(widget_materials_info_location, layout_materials_info[1]);
    f.render_widget(widget_materials_info_source, layout_materials_info[2]);
    f.render_widget(widget_materials_info_engineering, layout_materials_info[3]);
    f.render_widget(widget_materials_info_synthesis, layout_materials_info[4]);
}

// About --------------------------------------------------------------------------------------------------------------------------------------------------------------------

fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame) {
    // Layout definition
    let layout_about = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints(vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(chunk);

    // Widget definitions
    let widget_about_github = Paragraph::new("https://github.com/arne-fuchs/edcas-client")
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title("GitHub")
                .borders(Borders::TOP)
                .white(),
        );

    let widget_about_version = Paragraph::new(env!("CARGO_PKG_VERSION")).block(
        Block::default()
            .title("edcas version")
            .borders(Borders::TOP)
            .white(),
    );

    let widget_about_controls =
        Paragraph::new("Quit: Q, Change Tabs: q and e, Change System: Left and Right arrows, Change item selection: Up and Down arrows")
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .title("Controls")
                    .borders(Borders::TOP)
                    .white(),
            );

    // Render calls
    f.render_widget(widget_about_github, layout_about[1]);
    f.render_widget(widget_about_version, layout_about[2]);
    f.render_widget(widget_about_controls, layout_about[0]);
}
