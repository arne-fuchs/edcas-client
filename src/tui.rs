use core::f64;
use std::{error::Error, io};

use crossterm::{
    event::{self, Event::Key, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, style::Stylize, widgets::*};

use crate::edcas::EliteRustClient;
use crate::tui::about::tab_about;
use crate::tui::dockables::tab_dockables;
use crate::tui::explorer::tab_explorer;
use crate::tui::materials::tab_materials;
use crate::tui::mining::tab_mining;

mod about;
mod dockables;
mod explorer;
mod materials;
mod mining;

enum InputMode {
    Normal,
    Editing,
}

fn round_to_2(input: f64) -> f64 {
    (input * 100.0).round() / 100.0
}

fn round_to_4(input: f64) -> f64 {
    (input * 10000.0).round() / 10000.0
}

struct Search {
    pub input: String,
    pub cursor_position: usize,
}

impl Search {
    fn new() -> Search {
        Search {
            input: "".to_string(),
            cursor_position: 0,
        }
    }

    //Search Input materials
    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.cursor_position.saturating_sub(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.cursor_position.saturating_add(1);
        self.cursor_position = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        self.input.insert(self.cursor_position, new_char);

        self.move_cursor_right();
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.cursor_position != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.cursor_position;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.len())
    }
}

struct App<'a> {
    pub titles: Vec<&'a str>,             // tabs
    pub tab_index: usize,                 //
    pub body_list_state: ListState,       // explorer
    pub cargo_table_state: TableState,    // mining
    pub cargo_index: usize,               //
    pub prospector_list_state: ListState, //
    pub prospector_index: usize,          //
    pub material_index: usize,            // materials
    pub material_list_state: ListState,   //
    pub material_list_index: usize,       //
    pub search_input_mode: InputMode,     // user input
    pub material_search: Search,
    pub dockable_list_state: ListState,
    pub dockable_list_index: usize,
    pub dockable_search: Search,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Explorer", "Mining", "Materials", "Dockables", "About"],
            tab_index: 0,
            body_list_state: ListState::default(),
            prospector_list_state: ListState::default(),
            prospector_index: 0,
            cargo_table_state: TableState::default(),
            cargo_index: 0,
            material_index: 0,
            material_list_state: ListState::default(),
            material_list_index: 0,
            material_search: Search::new(),
            search_input_mode: InputMode::Normal,
            dockable_list_state: ListState::default(),
            dockable_list_index: 0,
            dockable_search: Search::new(),
        }
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
            && !client.explorer.systems[client.explorer.index]
                .body_list
                .is_empty()
        {
            client.explorer.systems[client.explorer.index].index =
                (client.explorer.systems[client.explorer.index].index + 1)
                    % client.explorer.systems[client.explorer.index]
                        .body_list
                        .len();
        }
    }

    pub fn previous_body(&mut self, client: &mut EliteRustClient) {
        if !client.explorer.systems.is_empty()
            && !client.explorer.systems[client.explorer.index]
                .body_list
                .is_empty()
        {
            if client.explorer.systems[client.explorer.index].index > 0 {
                client.explorer.systems[client.explorer.index].index -= 1;
            } else {
                client.explorer.systems[client.explorer.index].index = client.explorer.systems
                    [client.explorer.index]
                    .body_list
                    .len()
                    - 1;
            }
        }
    }

    /*
    pub fn next_info(&mut self, data_body_info: Vec<String>) {
        self.body_info_index = (self.body_info_index + 1) % data_body_info.len();
    }

    pub fn previous_info(&mut self, data_body_info: Vec<String>) {
        if self.body_info_index > 0 {
            self.body_info_index -= 1
        } else {
            self.body_info_index = data_body_info.len() - 1
        }
    }
    */

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

    pub fn next_prospector(&mut self, client: &EliteRustClient) {
        if !client.mining.prospectors.is_empty() {
            self.prospector_index = (self.prospector_index + 1) % client.mining.prospectors.len();
        }
    }

    pub fn previous_prospector(&mut self, client: &EliteRustClient) {
        if !client.mining.prospectors.is_empty() {
            if self.prospector_index > 0 {
                self.prospector_index -= 1;
            } else {
                self.prospector_index = client.mining.prospectors.len() - 1;
            }
        }
    }
    pub fn next_cargo(&mut self, client: &EliteRustClient) {
        let inventory_temp = &client.mining.cargo.lock().unwrap().inventory;
        if !inventory_temp.is_empty() {
            self.cargo_index = (self.cargo_index + 1) % inventory_temp.len();
        }
    }
    pub fn previous_cargo(&mut self, client: &EliteRustClient) {
        let inventory_temp = &client.mining.cargo.lock().unwrap().inventory;
        if !inventory_temp.is_empty() {
            if self.cargo_index > 0 {
                self.cargo_index -= 1;
            } else {
                self.cargo_index = inventory_temp.len() - 1;
            }
        }
    }

    // carriers
    pub fn next_dockable(&mut self, client: &mut EliteRustClient) {
        if !client.carrier.carriers.is_empty() {
            self.dockable_list_index =
                (self.dockable_list_index + 1) % client.carrier.carriers.len();
        }
    }

    pub fn previous_dockable(&mut self, client: &mut EliteRustClient) {
        if !client.carrier.carriers.is_empty() {
            if self.dockable_list_index > 0 {
                self.dockable_list_index -= 1
            } else {
                self.dockable_list_index = client.carrier.carriers.len() - 1;
            }
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

    // create edcas and run it
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
                                1 => app.next_prospector(&client),
                                2 => app.next_material_list(),
                                _ => {}
                            },
                            KeyCode::Left => match app.tab_index {
                                0 => app.previous_system(&mut client),
                                1 => app.previous_prospector(&client),
                                2 => app.previous_material_list(),
                                _ => {}
                            },
                            KeyCode::Down => match app.tab_index {
                                0 => app.next_body(&mut client),
                                1 => app.next_cargo(&client),
                                2 => app.next_material(&mut client),
                                3 => app.next_dockable(&mut client),
                                _ => {}
                            },
                            KeyCode::Up => match app.tab_index {
                                0 => app.previous_body(&mut client),
                                1 => app.previous_cargo(&client),
                                2 => app.previous_material(&mut client),
                                3 => app.previous_dockable(&mut client),
                                _ => {}
                            },
                            KeyCode::Char('i') => match app.tab_index {
                                2 => app.search_input_mode = InputMode::Editing,
                                3 => app.search_input_mode = InputMode::Editing,
                                _ => {}
                            },
                            _ => {}
                        },
                        InputMode::Editing => match key.code {
                            KeyCode::Char(to_insert) => match app.tab_index {
                                2 => app.material_search.enter_char(to_insert),
                                3 => app.dockable_search.enter_char(to_insert),
                                _ => {}
                            },
                            KeyCode::Backspace => match app.tab_index {
                                2 => app.material_search.delete_char(),
                                3 => app.dockable_search.delete_char(),

                                _ => {}
                            },
                            KeyCode::Left => match app.tab_index {
                                2 => app.material_search.move_cursor_left(),
                                3 => app.dockable_search.move_cursor_left(),
                                _ => {}
                            },
                            KeyCode::Right => match app.tab_index {
                                2 => app.material_search.move_cursor_right(),
                                3 => app.dockable_search.move_cursor_right(),
                                _ => {}
                            },
                            KeyCode::Esc => match app.tab_index {
                                2 => app.search_input_mode = InputMode::Normal,
                                3 => app.search_input_mode = InputMode::Normal,
                                _ => {}
                            },
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
        3 => tab_dockables(chunks[1], f, client, app),
        4 => tab_about(chunks[1], f),
        _ => unreachable!(),
    };
}
