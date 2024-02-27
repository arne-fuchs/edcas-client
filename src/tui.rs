use std::{error::Error, io};

use crossterm::{
    event::{self, Event::Key, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::app::{
    self,
    materials::{self, Material},
    EliteRustClient,
};

enum InputMode {
    Normal,
    Editing,
}

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub tab_index: usize,
    pub body_list_state: ListState,
    // list functionality of materials tab
    pub material_index: usize,
    pub material_list_state: ListState,
    // for switching between 3 lists
    pub material_list_index: usize,
    //user input
    pub search_input_mode: InputMode,
    pub search_cursor_position: usize,
    pub search_input: String,
    //shits to store the hashmap in
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Explorer", "Mining", "Materials", "About"],
            tab_index: 0,
            body_list_state: ListState::default(),
            material_index: 0,
            material_list_state: ListState::default(),
            material_list_index: 0,
            search_input_mode: InputMode::Normal,
            search_cursor_position: 0,
            search_input: "".to_string(),
        }
    }

    //serach Input
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.search_cursor_position.saturating_sub(1);
        self.search_cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.search_cursor_position.saturating_add(1);
        self.search_cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.search_input
            .insert(self.search_cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.search_cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.search_cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.search_input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.search_input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.search_input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.search_input.len())
    }

    fn reset_cursor(&mut self) {
        self.search_cursor_position = 0;
    }

    // functions for navigating scrollable elements
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
        self.material_index = (self.material_index + 1) % {
            match self.material_list_index {
                0 => client.materials.encoded.clone(),
                1 => client.materials.manufactured.clone(),
                2 => client.materials.raw.clone(),
                _ => client.materials.encoded.clone(),
            }
        }
        .len();
    }

    pub fn previous_material(&mut self, client: &mut EliteRustClient) {
        if self.material_index > 0 {
            self.material_index -= 1
        } else {
            self.material_index = {
                match self.material_list_index {
                    0 => client.materials.encoded.clone(),
                    1 => client.materials.manufactured.clone(),
                    2 => client.materials.raw.clone(),
                    _ => client.materials.encoded.clone(),
                }
            }
            .len()
                - 1;
        }
    }

    pub fn next_material_list(&mut self) {
        self.material_list_index = (self.material_list_index + 1) % 3;
    }

    pub fn previous_material_list(&mut self) {
        if self.material_list_index > 0 {
            self.material_list_index -= 1;
        } else {
            self.material_list_index = 2;
        }
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
                    match app.search_input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('Q') => return Ok(()),
                            KeyCode::Char('e') => app.next_tab(),
                            KeyCode::Char('q') => app.previous_tab(),
                            KeyCode::Right => match app.tab_index {
                                0 => app.next_system(&mut client),
                                2 => app.next_material_list(),
                                _ => {}
                            },
                            KeyCode::Left => match app.tab_index {
                                0 => app.previous_system(&mut client),
                                2 => app.previous_material_list(),
                                _ => {}
                            },
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
                            KeyCode::Char('i') => {
                                if app.tab_index == 2 {
                                    app.search_input_mode = InputMode::Editing;
                                }
                            }
                            _ => {}
                        },
                        // TODO: add keys for cursor navigation through signals list (do i need that?)
                        InputMode::Editing => match key.code {
                            KeyCode::Char(to_insert) => app.enter_char(to_insert),
                            KeyCode::Backspace => app.delete_char(),
                            KeyCode::Left => app.move_cursor_left(),
                            KeyCode::Right => app.move_cursor_right(),
                            KeyCode::Esc => app.search_input_mode = InputMode::Normal,
                            _ => {}
                        },
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

    // Widget definitions
    let widget_system_info = List::new(data_system_info).block(
        Block::default()
            .title(" System Info ")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    let widget_signal_list = List::new(data_signals_list).block(
        Block::default()
            .title(" Signals ")
            .borders(Borders::TOP)
            .bold()
            .white(),
    );

    let widget_body_list = List::new(data_body_list)
        .block(
            Block::default()
                .title(" Body List ")
                .borders(Borders::LEFT | Borders::TOP)
                .bold()
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_body_info = List::new(data_body_info).block(
        Block::default()
            .title(" Body Info ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );

    let widget_body_signals_list = List::new(data_body_signals_list).block(
        Block::default()
            .title(" Body Signals ")
            .borders(Borders::TOP | Borders::LEFT)
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
    // Selection from materials list (cursor and scrolling)
    app.material_list_state.select(Some(app.material_index));

    let mut material_vec_selected_sorted: Vec<&Material> = vec![];
    // Data processing
    let data_materials_dataset = match app.material_list_index {
        0 => client.materials.encoded.values().collect(),
        1 => client.materials.manufactured.values().collect(),
        2 => client.materials.raw.values().collect(),
        _ => vec![], // A Plug, so that rust wont complain about non-exhaustive match. Normally, any other values other than {0,1,2} would neven be accessible
    };
    let data_materials_dataset_name = match app.material_list_index {
        0 => "encoded",
        1 => "manufactured",
        2 => "raw",
        _ => "fallback, something went very wrong", // A Plug, so that rust wont complain about non-exhaustive match. Normally, any other values other than {0,1,2} would neven be accessible
    };

    for material_value in data_materials_dataset {
        if material_value
            .name_localised
            .to_lowercase()
            .contains(&app.search_input.to_lowercase())
            || material_value
                .name
                .to_lowercase()
                .contains(&app.search_input.to_lowercase())
        {
            material_vec_selected_sorted.push(material_value);
        }
    }

    // TODO: sort material_vec_selected_sorted (oder nicht?)
    material_vec_selected_sorted.sort_unstable_by_key(|sorting_key| {
        if sorting_key.name_localised != "null" {
            &sorting_key.name_localised
        } else {
            &sorting_key.name
        }
    });

    let mut data_materials_list_names: Vec<_> = vec![];
    let mut data_materials_list_count: Vec<_> = vec![];
    let mut data_materials_info = vec![];
    let mut data_materials_info_locations = vec![];
    let mut data_materials_info_description = "".to_string();
    let mut data_materials_info_sources = vec![];
    let mut data_materials_info_engineering = vec![];
    let mut data_materials_info_syntesis = vec![];

    //for search
    if !material_vec_selected_sorted.is_empty() {
        // check if pointer is out of bounds for list you are switching to. Set to list.len()-1 if it is.
        if app.material_index >= material_vec_selected_sorted.len() {
            app.material_index = material_vec_selected_sorted.len() - 1;
        }

        // bc the map is sorted, i can map index to key directly
        data_materials_list_names = material_vec_selected_sorted
            .iter()
            .map(|material| {
                if material.name_localised != "null" {
                    material.name_localised.clone()
                } else {
                    material.name.clone()
                }
            })
            .collect();

        data_materials_list_count = material_vec_selected_sorted
            .iter()
            .map(|material| [material.count.to_string(), material.maximum.to_string()].join("/"))
            .collect();

        data_materials_info = vec![
            [
                "Grade".to_string(),
                material_vec_selected_sorted[app.material_index]
                    .grade
                    .to_string(),
            ]
            .join(": "),
            [
                "Category".to_string(),
                material_vec_selected_sorted[app.material_index]
                    .category
                    .to_string(),
            ]
            .join(": "),
        ];

        data_materials_info_description = material_vec_selected_sorted[app.material_index]
            .description
            .clone();

        data_materials_info_locations = material_vec_selected_sorted[app.material_index]
            .locations
            .clone();

        data_materials_info_sources = material_vec_selected_sorted[app.material_index]
            .sources
            .clone();

        data_materials_info_engineering = material_vec_selected_sorted[app.material_index]
            .engineering
            .clone();

        data_materials_info_syntesis = material_vec_selected_sorted[app.material_index]
            .synthesis
            .clone();
    }

    // data on how long to make the layout
    let data_materials_info_locations_line_count: u16 = (data_materials_info_locations.len() + 2)
        .try_into()
        .unwrap();

    let data_materials_info_sources_line_count: u16 =
        (data_materials_info_sources.len() + 2).try_into().unwrap();

    let data_materials_info_engineering_line_count: u16 = (data_materials_info_engineering.len()
        + 2)
    .try_into()
    .unwrap();

    // Layout definitions
    let layout_materials = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(45), Constraint::Fill(1)])
        .split(chunk);

    let layout_materials_search_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(layout_materials[0]);

    let layout_materials_list = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(36), Constraint::Fill(1)])
        .split(layout_materials_search_list[1]);

    let layout_materials_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),                                          // Info
            Constraint::Length(12),                                         // Description
            Constraint::Length(data_materials_info_locations_line_count),   // Location
            Constraint::Length(data_materials_info_sources_line_count),     // Sources
            Constraint::Length(data_materials_info_engineering_line_count), // Engineering
            Constraint::Fill(1),                                            // Synthesis
        ])
        .split(layout_materials[1]);

    // Widget definitions
    // material_list field

    let widget_materials_search = Paragraph::new(app.search_input.clone()).block(
        Block::default()
            .borders(Borders::TOP)
            .white()
            .title(" Search "),
    );

    let widget_materials_list_names = List::new(data_materials_list_names)
        .block(
            Block::default()
                .title([" Materials: ", data_materials_dataset_name, " "].join(""))
                .borders(Borders::TOP)
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_materials_list_count =
        List::new(data_materials_list_count).block(Block::default().borders(Borders::TOP));

    // materials_info
    let widget_material_info = List::new(data_materials_info).block(
        Block::default()
            .title(" Material Information ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );
    let widget_materials_info_description = Paragraph::new(data_materials_info_description)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Description ")
                .bold()
                .borders(Borders::TOP | Borders::LEFT),
        );
    let widget_materials_info_location = List::new(data_materials_info_locations).block(
        Block::default()
            .title(" Location ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );
    let widget_materials_info_source = List::new(data_materials_info_sources).block(
        Block::default()
            .title(" Sources ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );
    let widget_materials_info_engineering = List::new(data_materials_info_engineering).block(
        Block::default()
            .title(" Engineering ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );
    let widget_materials_info_synthesis = List::new(data_materials_info_syntesis).block(
        Block::default()
            .title(" Synthesis ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );

    // Render calls
    f.render_widget(widget_materials_search, layout_materials_search_list[0]);
    f.render_stateful_widget(
        widget_materials_list_names,
        layout_materials_list[0],
        &mut app.material_list_state,
    );
    f.render_stateful_widget(
        widget_materials_list_count,
        layout_materials_list[1],
        &mut app.material_list_state,
    );

    f.render_widget(widget_material_info, layout_materials_info[0]);
    f.render_widget(widget_materials_info_description, layout_materials_info[1]);
    f.render_widget(widget_materials_info_location, layout_materials_info[2]);
    f.render_widget(widget_materials_info_source, layout_materials_info[3]);
    f.render_widget(widget_materials_info_engineering, layout_materials_info[4]);
    f.render_widget(widget_materials_info_synthesis, layout_materials_info[5]);
    // make cursor visible for input
    match app.search_input_mode {
        InputMode::Normal =>
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            {}

        InputMode::Editing => {
            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            f.set_cursor(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                layout_materials_search_list[0].x + app.search_cursor_position as u16,
                // Move one line down, from the border to the input line
                layout_materials_search_list[0].y + 1,
            )
        }
    }
}
// About --------------------------------------------------------------------------------------------------------------------------------------------------------------------

fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame) {
    // data here if needed
    let data_controls_list = vec![
        "Quit: Q, Change Tabs: q and e",
        "Change System/Material List: Left and Right arrows",
        "Change Body/Material selection: Up and Down arrows",
        "Search: i",
        "Quit Search: esc",
    ];

    let data_controls_list_size: u16 = (data_controls_list.len() + 2).try_into().unwrap();

    // Layout definition
    let layout_about = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints(vec![
            Constraint::Length(data_controls_list_size), // TODO: hardcode correct value
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(chunk);

    // Widget definitions
    let widget_about_github = Paragraph::new("https://github.com/arne-fuchs/edcas-client")
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" GitHub ")
                .borders(Borders::TOP)
                .white(),
        );

    let widget_about_version = Paragraph::new(env!("CARGO_PKG_VERSION")).block(
        Block::default()
            .title(" edcas version ")
            .borders(Borders::TOP)
            .white(),
    );

    let widget_about_controls = List::new(data_controls_list).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::TOP)
            .white(),
    );

    // Render calls
    f.render_widget(widget_about_github, layout_about[1]);
    f.render_widget(widget_about_version, layout_about[2]);
    f.render_widget(widget_about_controls, layout_about[0]);
}
