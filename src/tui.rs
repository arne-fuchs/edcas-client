use core::f64;
use std::{cmp::Ordering, error::Error, io};

use crossterm::{
    event::{self, Event::Key, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, style::Stylize, widgets::*};

<<<<<<< HEAD
use crate::edcas::{materials::Material, mining::MiningMaterial, EliteRustClient};
=======
use crate::app::{
    explorer::structs::{self, BodyType},
    materials::Material,
    mining::MiningMaterial,
    EliteRustClient,
};
>>>>>>> 0208ae9 (basic body info implementations)

// TODO: DONE signals_scanned/all_signals (gauge and text)
// TODO: DONE signal threat (in system_signals)
// TODO: DONE body signal count (in body_signals)
// TODO: DONE Body signals in system tree
// TODO: styling (probably rewrite everything to use Span, Line and Text)
// TODO: body info (the api for that is not ready yet)

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

    fn filter_by_input(&mut self, list: Vec<String>) -> Vec<String> {
        let mut list_selected: Vec<String> = vec![];
        for value in list {
            if value.to_lowercase().contains(&self.input.to_lowercase()) {
                list_selected.push(value);
            }
        }
        list_selected
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
    pub carrier_list_state: ListState,
    pub carrier_list_index: usize,
    pub carrier_search: Search,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Explorer", "Mining", "Materials", "Carrier", "About"],
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
            carrier_list_state: ListState::default(),
            carrier_list_index: 0,
            carrier_search: Search::new(),
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
    pub fn next_carrier(&mut self, client: &mut EliteRustClient) {
        if !client.carrier.carriers.is_empty() {
            self.carrier_list_index = (self.carrier_list_index + 1) % client.carrier.carriers.len();
        }
    }

    pub fn previous_carrier(&mut self, client: &mut EliteRustClient) {
        if !client.carrier.carriers.is_empty() {
            if self.carrier_list_index > 0 {
                self.carrier_list_index -= 1
            } else {
                self.carrier_list_index = client.carrier.carriers.len() - 1;
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
                                3 => app.next_carrier(&mut client),
                                _ => {}
                            },
                            KeyCode::Up => match app.tab_index {
                                0 => app.previous_body(&mut client),
                                1 => app.previous_cargo(&client),
                                2 => app.previous_material(&mut client),
                                3 => app.previous_carrier(&mut client),
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
                                3 => app.carrier_search.enter_char(to_insert),
                                _ => {}
                            },
                            KeyCode::Backspace => match app.tab_index {
                                2 => app.material_search.delete_char(),
                                3 => app.carrier_search.delete_char(),
                                _ => {}
                            },
                            KeyCode::Left => match app.tab_index {
                                2 => app.material_search.move_cursor_left(),
                                3 => app.carrier_search.move_cursor_left(),
                                _ => {}
                            },
                            KeyCode::Right => match app.tab_index {
                                2 => app.material_search.move_cursor_right(),
                                3 => app.carrier_search.move_cursor_right(),
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
        3 => tab_carrier(chunks[1], f, client, app),
        4 => tab_about(chunks[1], f),
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
    let mut data_system_info = vec![Row::new(vec!["no data".to_string()])];
    let mut data_system_name = Span::from("no data").light_red();
    let mut data_signals_list = vec![Row::new(vec!["no data".to_string()])];
    let mut data_body_list: Vec<Line> = vec![Line::styled("no data", Style::default().light_red())];
    //let mut data_body_signals_list = vec![Row::new(vec!["no data".to_string()])];
    let mut data_body_info: Vec<Row> =
        vec![Row::new(vec!["no data".to_string()]).style(Style::default().light_red())];
    let mut data_body_name = Span::from("no data").light_red();
    let mut data_system_gauge_scanned: i32 = 0;
    let mut data_system_gauge_all: i32 = 0;
    let mut data_system_gauge: f64 = 0.0 / 1.0;
    let mut data_planet_signals: Vec<Row> = vec![Row::new(vec!["no data".light_red()])]; //client.explorer.systems[index].planet_signals[index] (body_name, body_id, Vec signals)
    let mut additional_length_data_body_info: usize = 0;

    // Checks to not crash everything if list is empty
    // Data acquisition
    if !client.explorer.systems.is_empty() {
        data_system_name = Span::from(
            client.explorer.systems[client.explorer.index]
                .name
                .to_string(),
        );
        data_system_info = vec![
            //Row::new(vec!["Name".to_string(),client.explorer.systems[client.explorer.index].name.clone(),]),
            Row::new(vec![
                "Allegiance".to_string(),
                client.explorer.systems[client.explorer.index]
                    .allegiance
                    .clone(),
            ]),
            Row::new([
                "Economy".to_string(),
                client.explorer.systems[client.explorer.index]
                    .economy_localised
                    .clone(),
            ]),
            Row::new([
                "2. Economy".to_string(),
                client.explorer.systems[client.explorer.index]
                    .second_economy_localised
                    .clone(),
            ]),
            Row::new([
                "Government".to_string(),
                client.explorer.systems[client.explorer.index]
                    .government_localised
                    .clone(),
            ]),
            Row::new([
                "Security".to_string(),
                client.explorer.systems[client.explorer.index]
                    .security_localised
                    .clone(),
            ]),
            Row::new([
                "Population".to_string(),
                client.explorer.systems[client.explorer.index]
                    .population
                    .clone(),
            ]),
            Row::new([
                "Bodies".to_string(),
                client.explorer.systems[client.explorer.index]
                    .body_count
                    .clone(),
            ]),
            Row::new([
                "Non-bodies".to_string(),
                client.explorer.systems[client.explorer.index]
                    .non_body_count
                    .clone(),
            ]),
        ];

        data_signals_list = client.explorer.systems[client.explorer.index]
            .signal_list
            .iter()
            .map(|signal| Row::new(vec![signal.name.to_string(), signal.threat.to_string()]))
            .collect::<Vec<Row>>();

        if !client.explorer.systems[client.explorer.index]
            .planet_signals
            .is_empty()
        {
            data_planet_signals.clear();
            for planet_signal in &client.explorer.systems[client.explorer.index].planet_signals {
                for signal in &planet_signal.signals {
                    let signal_type: &str = if signal.type_localised != "null" {
                        signal.type_localised.as_str()
                    } else {
                        signal.r#type.as_str()
                    };

                    data_planet_signals.push(Row::new(vec![
                        planet_signal
                            .body_name
                            .trim_start_matches(
                                &client.explorer.systems[client.explorer.index].name,
                            )
                            .into(),
                        match signal_type {
                            "Biological" => signal_type.light_green(),
                            "Geological" => signal_type.magenta(),
                            _ => signal_type.into(),
                        },
                        signal.count.to_string().into(),
                    ]))
                }
            }
        } else {
            data_planet_signals.clear();
        }

        if client.explorer.systems[client.explorer.index].non_body_count != "N/A"
            && client.explorer.systems[client.explorer.index].body_count != "N/A"
        {
            data_system_gauge_scanned = client.explorer.systems[client.explorer.index]
                .body_list
                .len() as i32;

            data_system_gauge_all = client.explorer.systems[client.explorer.index]
                .non_body_count
                .parse::<i32>()
                .unwrap()
                + client.explorer.systems[client.explorer.index]
                    .body_count
                    .parse::<i32>()
                    .unwrap();

            if data_system_gauge_scanned > data_system_gauge_all {
                data_system_gauge_all = data_system_gauge_scanned; //shouldnt be the case
                                                                   //but it did crash one time i used
                                                                   //system signals as scanned
            }

            data_system_gauge = f64::from(data_system_gauge_scanned)
                / if data_system_gauge_all != 0 {
                    f64::from(data_system_gauge_all)
                } else {
                    1.0 //just to be sure
                };
        }

        if !client.explorer.systems[client.explorer.index]
            .body_list
            .is_empty()
        {
            //preparet to implement fancy tree, too dumb rn to do it.
            data_body_list.clear();
            for body in client.explorer.systems[client.explorer.index]
                .body_list
                .iter()
                .rev()
            {
                let mut space_string = "".to_string();

                for i in 0..body.get_parents().len() {
                    if i < body.get_parents().len() - 1 {
                        space_string.push_str("│  ")
                    } else {
                        space_string.push_str("│  ");
                    }
                }
                space_string.push_str(body.get_name().as_str());
                let signals_type_string: Vec<String> = body
                    .get_signals()
                    .iter()
                    .map(|body_signal| {
                        if body_signal.type_localised != "null" {
                            body_signal.type_localised.to_string()
                        } else {
                            body_signal.r#type.to_string()
                        }
                    })
                    .collect();
                data_body_list.push(
                    vec![
                        space_string.fg(Color::White),
                        " ".into(),
                        signals_type_string.join(" ").light_green().italic(),
                    ]
                    .into(),
                )
            }

            data_body_list.reverse();

            data_body_info = match client.explorer.systems[client.explorer.index].body_list
                [client.explorer.systems[client.explorer.index].index]
                .get_body()
            {
                BodyType::Star(star_body) => {
                    data_body_name = Span::from(star_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string(),star_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), star_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),star_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), star_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), star_body.body_id.to_string()]),
                        //Row::new(vec!["System".to_string(),star_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(), star_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(star_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec!["Type".to_string(), star_body.star_type.to_string()]),
                        Row::new(vec!["Subclass".to_string(), star_body.subclass.to_string()]),
                        Row::new(vec![
                            "Solar Mass".to_string(),
                            round_to_4(star_body.stellar_mass).to_string(),
                        ]),
                        Row::new(vec![
                            "Solar Radius".to_string(),
                            round_to_4(star_body.radius / 695508000.0).to_string(),
                        ]),
                        Row::new(vec![
                            "Abs. Magnitude".to_string(),
                            star_body.absolute_magnitude.to_string(),
                        ]),
                        Row::new(vec![
                            "Age".to_string(),
                            [star_body.age_my.to_string(), "M.Years".to_string()].join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Temp".to_string(),
                            [star_body.surface_temperature.to_string(), "K".to_string()].join(" "),
                        ]),
                        Row::new(vec![
                            "Luminosity".to_string(),
                            star_body.luminosity.to_string(),
                        ]),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            match star_body.semi_major_axis {
                                Some(sma) => [
                                    round_to_4(sma / 149597870700.0).to_string(),
                                    "AU".to_string(),
                                ]
                                .join(" "),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Eccentricity".to_string(),
                            match star_body.eccentricity {
                                Some(sma) => round_to_4(sma).to_string(),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Orb. Inclanation".to_string(),
                            match star_body.orbital_inclination {
                                Some(sma) => {
                                    [round_to_2(sma).to_string(), "°".to_string()].join("")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            match star_body.periapsis {
                                Some(sma) => {
                                    [round_to_2(sma).to_string(), "°".to_string()].join("")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Orbital Period".to_string(),
                            match star_body.orbital_period {
                                Some(sma) => {
                                    [round_to_2(sma / 86400.0).to_string(), "D".to_string()]
                                        .join(" ")
                                }
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            match star_body.mean_anomaly {
                                Some(sma) => round_to_4(sma).to_string(),
                                None => "no data".to_string(),
                            },
                        ]),
                        Row::new(vec![
                            "Rot. Period".to_string(),
                            [
                                round_to_2(star_body.rotation_period / 86400.0).to_string(),
                                "D".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Axial Tilt".to_string(),
                            [
                                round_to_2(star_body.axial_tilt * 180.0 / f64::consts::PI)
                                    .to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "was Discovered".to_string(),
                            if star_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "was Mapped".to_string(),
                            if star_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Asteroid Rings".to_string(),
                            star_body.asteroid_rings.len().to_string(),
                        ]),
                    ]
                }

                BodyType::BeltCluster(belt_body) => {
                    data_body_name = Span::from(belt_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string()belt_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), belt_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),belt_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), belt_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), belt_body.body_id.to_string()]),
                        //Row::new(vec!["System".to_string(),belt_body.star_system.to_string(),]),
                        //Row::new(vec!["Parents".to_string(),belt_body.parents.len().to_string(),]),
                        //Row::new(vec!["System Address".to_string(), belt_body.system_address.to_string(),]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(belt_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if belt_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if belt_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                    ]
                }
                BodyType::Planet(planet_body) => {
                    data_body_name = Span::from(planet_body.body_name.to_string());
                    vec![
                        // Row::new(vec!["Timestamp".to_string(),planet_body.timestamp.to_string()]),
                        // Row::new(vec!["Event".to_string(), planet_body.event.to_string()]),
                        // Row::new(vec!"Scan Type".to_string()planet_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), planet_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), planet_body.body_id.to_string()]),
                        //Row::new(vec!["Parents".to_string(),planet_body.parents.len().to_string(),]),
                        //Row::new(vec!["System".to_string(),planet_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(),planet_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(planet_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        //Row::new(vec!["Tidal Lock".to_string(),planet_body.tidal_lock.to_string(),]),
                        Row::new(vec![
                            "Terraform State".to_string(),
                            planet_body.terraform_state.to_string(),
                        ]),
                        Row::new(vec![
                            "Class".to_string(),
                            planet_body.planet_class.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere".to_string(),
                            planet_body.atmosphere.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere Type".to_string(),
                            planet_body.atmosphere_type.to_string(),
                        ]),
                        Row::new(vec![
                            "Atmosphere Comp.".to_string(),
                            planet_body
                                .atmosphere_composition
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percent.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ])
                        .height(
                            if planet_body.atmosphere_composition.is_empty() {
                                1
                            } else {
                                additional_length_data_body_info +=
                                    planet_body.atmosphere_composition.len() - 1;
                                planet_body.atmosphere_composition.len() as u16
                            },
                        ),
                        Row::new(vec![
                            "Volcanism".to_string(),
                            planet_body.volcanism.to_string(),
                        ]),
                        Row::new(vec![
                            "Earth Masses".to_string(),
                            round_to_4(planet_body.mass_em).to_string(),
                        ]),
                        Row::new(vec![
                            "Radius".to_string(),
                            [
                                round_to_2(planet_body.radius / 1000.0).to_string(),
                                "km".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Gravity".to_string(),
                            [
                                round_to_2(planet_body.surface_gravity * 0.1).to_string(),
                                "G".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Temp.".to_string(),
                            [
                                round_to_2(planet_body.surface_temperature).to_string(),
                                "K".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Surface Pressure".to_string(),
                            [
                                round_to_4(planet_body.surface_pressure / 101325.0).to_string(),
                                //Pascal to atmosphere
                                "Atm".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Landable".to_string(),
                            if planet_body.landable {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Materials".to_string(),
                            planet_body
                                .materials
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percentage.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ]),
                        Row::new(vec![
                            "Composition".to_string(),
                            planet_body
                                .composition
                                .iter()
                                .map(|mat| {
                                    [
                                        mat.name.to_string(),
                                        " ".to_string(),
                                        mat.percentage.to_string(),
                                        "%".to_string(),
                                    ]
                                    .join("")
                                })
                                .collect::<Vec<String>>()
                                .join("\n"),
                        ])
                        .height(if planet_body.composition.is_empty() {
                            1
                        } else {
                            additional_length_data_body_info += planet_body.composition.len() - 1;
                            planet_body.composition.len() as u16
                        }),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            [
                                round_to_2(planet_body.semi_major_axis / 149597870700.0)
                                    .to_string(),
                                "AU".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Eccentricity".to_string(),
                            round_to_4(planet_body.eccentricity).to_string(),
                        ]),
                        Row::new(vec![
                            "Orb. Inclanation".to_string(),
                            [
                                round_to_2(planet_body.orbital_inclination).to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            [
                                round_to_2(planet_body.periapsis).to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Orbital Period".to_string(),
                            [
                                round_to_2(planet_body.orbital_period / 86400.0).to_string(),
                                "D".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Ascending Node".to_string(),
                            round_to_2(planet_body.ascending_node).to_string(),
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            round_to_2(planet_body.mean_anomaly).to_string(),
                        ]),
                        Row::new(vec![
                            "Rot. Period".to_string(),
                            [
                                round_to_2(planet_body.rotation_period / 86400.0).to_string(),
                                if planet_body.tidal_lock {
                                    "D [Tidally Locked]".to_string()
                                } else {
                                    "D".to_string()
                                },
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Axial Tilt".to_string(),
                            [
                                round_to_2(planet_body.axial_tilt * 180.0 / f64::consts::PI)
                                    .to_string(),
                                "°".to_string(),
                            ]
                            .join(""),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if planet_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if planet_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Reserve Level".to_string(),
                            planet_body.reserve_level.to_string(),
                        ]),
                        Row::new(vec![
                            "Rings".to_string(),
                            planet_body.asteroid_rings.len().to_string(),
                        ]),
                        //Row::new(vec!["Signals".to_string(),planet_body.planet_signals.len().to_string(),]),
                    ]
                }
                BodyType::Ring(ring_body) => {
                    data_body_name = Span::from(ring_body.body_name.to_string());
                    vec![
                        //Row::new(vec!["Timestamp".to_string(),ring_body.timestamp.to_string(),]),
                        //Row::new(vec!["Event".to_string(), ring_body.event.to_string()]),
                        //Row::new(vec!["Scan Type".to_string(),ring_body.scan_type.to_string(),]),
                        //Row::new(vec!["Name".to_string(), ring_body.body_name.to_string()]),
                        //Row::new(vec!["ID".to_string(), ring_body.body_id.to_string()]),
                        //Row::new(vec!["Parents".to_string(),ring_body.parents.len().to_string(),]),
                        //Row::new(vec!["System".to_string(),ring_body.star_system.to_string(),]),
                        //Row::new(vec!["".to_string(), ring_body.system_address.to_string()]),
                        Row::new(vec![
                            "Distance".to_string(),
                            [
                                round_to_2(ring_body.distance_from_arrival_ls).to_string(),
                                "ls".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Semi major Axis".to_string(),
                            [
                                round_to_4(ring_body.semi_major_axis / 149597870700.0).to_string(),
                                "AU".to_string(),
                            ]
                            .join(" "),
                        ]),
                        Row::new(vec![
                            "Arg of Periapsis".to_string(),
                            [round_to_2(ring_body.periapsis).to_string(), "°".to_string()].join(""),
                        ]),
                        Row::new(vec![
                            "Ascending Node".to_string(),
                            round_to_2(ring_body.ascending_node).to_string(),
                        ]),
                        Row::new(vec![
                            "Mean Anomaly".to_string(),
                            round_to_2(ring_body.mean_anomaly).to_string(),
                        ]),
                        Row::new(vec![
                            "Discovered".to_string(),
                            if ring_body.was_discovered {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        Row::new(vec![
                            "Mapped".to_string(),
                            if ring_body.was_mapped {
                                "yes".to_string()
                            } else {
                                "no".to_string()
                            },
                        ]),
                        /*
                        Row::new(vec!["Signals".to_string(),ring_body.ring_signals.iter(.map(|sig| {[{
                                            if sig.type_localised != "null" {
                                                sig.type_localised.to_string()
                                            } else {
                                                sig.r#type.to_string()
                                            }},
                                        sig.count.to_string(),].join(" ")}).collect::<Vec<String>>().join("\n"),])
                        .height(if ring_body.ring_signals.is_empty() {1} else {
                            additional_length_data_body_info += ring_body.ring_signals.len() - 1;
                            ring_body.ring_signals.len() as u16
                        }),
                        */
                    ]
                }
            };

            // Selection from body_list (cursor and scrolling)
            app.body_list_state
                .select(Some(client.explorer.systems[client.explorer.index].index));
            /*
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
                        Row::new(vec![
                            if body_signal.type_localised != "null" {
                                body_signal.type_localised.clone()
                            } else {
                                body_signal.r#type.clone()
                            },
                            body_signal.count.to_string(),
                        ])
                    })
                    .collect::<Vec<Row>>();
            } else {
                data_body_signals_list =
                    vec![Row::new(vec!["no signals".to_string(), "".to_string()])];
            }*/
        }
    }

    // Layout definitions
    // general layout
    let layout_explorer = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Horizontal)
        .constraints(vec![
            Constraint::Fill(2),
            Constraint::Fill(3),
            Constraint::Fill(3),
        ])
        .split(chunk);

    // layout of "systems" Panel
    let layout_system = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Fill(1)])
        .split(layout_explorer[0]);

    // layout of "body information" panel
    let layout_body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(
                data_body_info.len() as u16 + additional_length_data_body_info as u16 + 1,
            ),
            Constraint::Fill(1), //Min(20),
        ])
        .split(layout_explorer[2]);

    // layout of system inforamtion
    let layout_system_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(1)])
        .split(layout_system[0]);

    // Widget definitions
    let widget_system_info = Table::new(
        data_system_info,
        [Constraint::Length(10), Constraint::Fill(1)],
    )
    .block(
        Block::default()
            .title(Line::from(vec![
                Span::from(" "),
                data_system_name,
                Span::from(" "),
            ]))
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );

    let widget_system_gauge = LineGauge::default()
        .block(Block::default().borders(Borders::LEFT))
        .line_set(symbols::line::THICK)
        .gauge_style(Style::default().fg(Color::LightRed).bg(Color::Black))
        .label(format!(
            "{data_system_gauge_scanned}/{data_system_gauge_all}"
        ))
        .ratio(data_system_gauge);

    let widget_signal_list = Table::new(
        data_signals_list,
        [Constraint::Fill(1), Constraint::Length(2)],
    )
    .header(Row::new(vec!["Name", "TL"]))
    .block(
        Block::default()
            .title(" Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );

    let widget_body_list = List::new(data_body_list) //List::new(data_body_list)
        .block(
            Block::default()
                .title(" Body List ")
                .borders(Borders::LEFT | Borders::TOP)
                .bold()
                .white(),
        )
        .highlight_style(Style::default().bold().on_dark_gray());

    let widget_body_info = Table::new(
        data_body_info,
        vec![Constraint::Length(16), Constraint::Fill(1)],
    )
    .block(
        Block::default()
            .title(Line::from(vec![
                Span::from(" Body: "),
                data_body_name,
                Span::from(" "),
            ]))
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );
    /*
    let widget_body_signals_list = Table::new(
        data_body_signals_list,
        [Constraint::Fill(1), Constraint::Length(6)],
    )
    .header(Row::new(vec!["Name", "Count"]))
    .block(
        Block::default()
            .title(" Body Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold()
            .white(),
    );*/

    // TODO: make table scrollable how?
    let widget_planet_signals_list = Table::new(
        data_planet_signals,
        [
            Constraint::Length(8),
            Constraint::Fill(1),
            Constraint::Length(3),
        ],
    )
    .header(Row::new(vec!["Body", "Signal", "#"]))
    .block(
        Block::default()
            .title(" Body Signals ")
            .borders(Borders::TOP | Borders::LEFT)
            .bold(),
    );

    // render calls
    f.render_widget(widget_system_info, layout_system_info[0]);
    f.render_widget(widget_system_gauge, layout_system_info[1]);
    f.render_widget(widget_signal_list, layout_system[1]);

    f.render_stateful_widget(
        widget_body_list,
        layout_explorer[1],
        &mut app.body_list_state,
    );

    f.render_widget(widget_body_info, layout_body[0]);
    f.render_widget(widget_planet_signals_list, layout_body[1])
    //f.render_widget(widget_body_signals_list, layout_body[1]);
}

// Mining --------------------------------------------------------------------------------------------------------------------------------------------------------------------

//TODO: function that constructs a that text thing, map to the .prospectors vector

fn data_prospector_text(
    mining_material: &[MiningMaterial],
    mining_content: &String,
    remaining: &f64,
) -> String {
    let mut return_string: Vec<String> = vec![];

    for material in mining_material {
        return_string.push(
            [
                "│".to_string(),
                if material.name_localised != "null" {
                    material.name_localised.to_owned()
                } else {
                    material.name.to_owned()
                },
                material.proportion.to_string(),
                material.buy_price.to_string(),
            ]
            .join(" "),
        );
    }

    [
        [
            "┌ ".to_string(),
            mining_content.to_owned(),
            " ───────────────────".to_string(),
        ]
        .join(""),
        [
            "│ ".to_string(),
            "remaining: ".to_string(),
            remaining.to_string(),
        ]
        .join(""),
        [
            "│ ".to_string(),
            "Name ".to_string(),
            "Content ".to_string(),
            "Price ".to_string(),
        ]
        .join(""),
        return_string.join("\n"),
        "│".to_string(),
    ]
    .join("\n")
}

fn tab_mining(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    //data
    // should i implement a struct in a fucntion that gets called about 30times per second? probably not
    impl MiningMaterial {
        fn default() -> MiningMaterial {
            MiningMaterial {
                name: "no data".to_string(),
                name_localised: "no data".to_string(),
                buy_price: 0.0,
                proportion: 0.0,
            }
        }
    }

    let default_material: MiningMaterial = MiningMaterial::default();

    let mut data_prospector_list: Vec<_> = vec![data_prospector_text(
        &[default_material],
        &"no data".to_string(),
        &0.0,
    )];

    if !&client.mining.prospectors.is_empty() {
        for prosp in &client.mining.prospectors {
            data_prospector_list.push(data_prospector_text(
                &prosp.materials,
                &prosp.content_localised,
                &prosp.remaining,
            ))
        }
        //*edcas.prospector_list_state.offset_mut() = edcas.prospector_index;
        app.prospector_list_state.select(Some(app.prospector_index));
        // would rather update it near cargo_table_state, but dont want to write another if  specifically for it
    }

    data_prospector_list = client
        .mining
        .prospectors
        .iter()
        .map(|prosp| {
            data_prospector_text(&prosp.materials, &prosp.content_localised, &prosp.remaining)
        })
        .collect();

    let mut data_cargo_rows: Vec<Row> = vec![];

    for cargo_element in &client.mining.cargo.lock().unwrap().inventory {
        data_cargo_rows.push(Row::new(vec![
            if cargo_element.name_localised != "null" {
                cargo_element.name_localised.clone()
            } else {
                cargo_element.name.clone()
            },
            cargo_element.count.to_string(),
            cargo_element.buy_price.to_string(),
            cargo_element.highest_sell_price.to_string(),
            cargo_element.highest_sell_system.clone(),
            cargo_element.highest_sell_station.clone(),
        ]));
    }

    // update table state if table is not empty
    if !data_cargo_rows.is_empty() {
        app.cargo_table_state.select(Some(app.cargo_index));
    }

    //layout
    let layout_mining = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Fill(1)])
        .split(chunk);

    //table
    let cargo_spacing = [
        Constraint::Length(12), // name
        Constraint::Length(4),  // name
        Constraint::Length(10), // avg buy
        Constraint::Length(10), // highst sel
        Constraint::Length(14), // avg buy
        Constraint::Fill(1),    // station
    ];

    //widgets
    let widget_mining_prospector = List::new(data_prospector_list)
        .block(
            Block::default()
                .title(" Prospector ")
                .borders(Borders::TOP | Borders::LEFT),
        )
        .highlight_style(Style::default().white().on_dark_gray());

    let cargo_header = Row::new(vec![
        "Name",
        "Qty",
        "Avg. buy",
        "Hig. sell",
        "System",
        "Station",
    ]);

    let widget_mining_cargo = Table::new(data_cargo_rows, cargo_spacing)
        .header(cargo_header)
        .block(
            Block::default()
                .title(" Cargo ")
                .borders(Borders::LEFT | Borders::TOP),
        )
        .highlight_style(Style::default().on_dark_gray());

    //rendering
    f.render_stateful_widget(
        widget_mining_prospector,
        layout_mining[0],
        &mut app.prospector_list_state,
    ); // prospector
    f.render_stateful_widget(
        widget_mining_cargo,
        layout_mining[1],
        &mut app.cargo_table_state,
    );
    // cargo
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
    let data_materials_dataset: Vec<&Material> = match app.material_list_index {
        0 => client.materials.encoded.values().collect(),
        1 => client.materials.manufactured.values().collect(),
        2 => client.materials.raw.values().collect(),
        _ => unreachable!(),
    };
    let data_materials_dataset_name = match app.material_list_index {
        0 => "encoded",
        1 => "manufactured",
        2 => "raw",
        _ => unreachable!(),
    };

    // do the search with Search::filter_by_input
    for material_value in data_materials_dataset {
        if material_value
            .name_localised
            .to_lowercase()
            .contains(&app.material_search.input.to_lowercase())
            || material_value
                .name
                .to_lowercase()
                .contains(&app.material_search.input.to_lowercase())
        {
            material_vec_selected_sorted.push(material_value);
        }
    }

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
            app.material_index %= material_vec_selected_sorted.len();
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

    let widget_materials_search = Paragraph::new(app.material_search.input.clone()).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT)
            .white()
            .title(" Search "),
    );

    let widget_materials_list_names = List::new(data_materials_list_names)
        .block(
            Block::default()
                .title([" Materials: ", data_materials_dataset_name, " "].join(""))
                .borders(Borders::TOP | Borders::LEFT)
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
                layout_materials_search_list[0].x + 1 + app.material_search.cursor_position as u16,
                // Move one line down, from the border to the input line
                layout_materials_search_list[0].y + 1,
            )
        }
    }
}

fn tab_carrier(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    app.carrier_list_state.select(Some(app.carrier_list_index));

    let mut data_carrier_list: Vec<String> = vec!["no data".to_string()];
    let mut data_carrier_list_selected: Vec<String> = data_carrier_list;
    let mut data_carrier_info_location = "no data".to_string();
    let mut data_carrier_info_destination = "no data".to_string();
    let mut data_carrier_info_modules: Vec<String> = vec!["no data".to_string()];
    let mut data_carrier_info_other = "no data".to_string();

    if !client.carrier.carriers.is_empty() {
        data_carrier_list = client
            .carrier
            .carriers
            .iter()
            .map(|carrier| [carrier.name.to_string(), carrier.callsign.to_string()].join(" - "))
            .collect::<Vec<String>>();

        data_carrier_list_selected = app.carrier_search.filter_by_input(data_carrier_list);

        match app
            .carrier_list_index
            .cmp(&data_carrier_list_selected.len())
        {
            Ordering::Greater => app.carrier_list_index = data_carrier_list_selected.len() - 1, // thats no good, store selected list or its length in app.
            Ordering::Equal => app.carrier_list_index = 0,
            _ => {}
        };

        data_carrier_info_location = [
            client.carrier.carriers[app.carrier_list_index]
                .current_system
                .to_string(),
            client.carrier.carriers[app.carrier_list_index]
                .current_body
                .to_string(),
        ]
        .join(" - ");

        data_carrier_info_destination = [
            client.carrier.carriers[app.carrier_list_index]
                .next_system
                .to_string(),
            " - ".to_string(),
            client.carrier.carriers[app.carrier_list_index]
                .next_body
                .to_string(),
            "\n".to_string(),
            client.carrier.carriers[app.carrier_list_index]
                .departure
                .to_string(),
        ]
        .join("");

        data_carrier_info_modules = client.carrier.carriers[app.carrier_list_index]
            .services
            .split(',')
            .map(|f| f.to_string())
            .collect::<Vec<String>>();

        data_carrier_info_other = [
            "Allow notorious:".to_string(),
            client.carrier.carriers[app.carrier_list_index]
                .allow_notorious
                .to_string(),
            "\nDocking Access:".to_string(),
            client.carrier.carriers[app.carrier_list_index]
                .docking_access
                .to_string(),
        ]
        .join(" ");
    }

    let layout_carrier = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(45), Constraint::Fill(1)])
        .split(chunk);

    let layout_carrier_search_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(layout_carrier[0]);

    let layout_carrier_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),                                          // Location
            Constraint::Length(4),                                          // Destination
            Constraint::Length(data_carrier_info_modules.len() as u16 + 2), // Modules
            Constraint::Fill(1),                                            // Other
        ])
        .split(layout_carrier[1]);

    // Widget definitions
    let widget_carrier_search = Paragraph::new(app.carrier_search.input.clone()).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT)
            .white()
            .title(" Search "),
    );

    let widget_carrier_list = List::new(data_carrier_list_selected)
        .block(
            Block::default()
                .title(" Known Carriers ")
                .borders(Borders::TOP | Borders::LEFT)
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_carrier_info_location = Paragraph::new(data_carrier_info_location)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Location ")
                .bold()
                .borders(Borders::TOP | Borders::LEFT),
        );
    let widget_carrier_info_destination = Paragraph::new(data_carrier_info_destination)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Jump Destination ")
                .bold()
                .borders(Borders::TOP | Borders::LEFT),
        );
    let widget_carrier_info_modules = List::new(data_carrier_info_modules).block(
        Block::default()
            .title(" Available Services ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );
    let widget_carrier_info_other = Paragraph::new(data_carrier_info_other).block(
        Block::default()
            .title(" Other ")
            .bold()
            .borders(Borders::TOP | Borders::LEFT),
    );

    // Render calls
    f.render_widget(widget_carrier_search, layout_carrier_search_list[0]);
    f.render_stateful_widget(
        widget_carrier_list,
        layout_carrier_search_list[1],
        &mut app.carrier_list_state,
    );

    f.render_widget(widget_carrier_info_location, layout_carrier_info[0]);
    f.render_widget(widget_carrier_info_destination, layout_carrier_info[1]);
    f.render_widget(widget_carrier_info_modules, layout_carrier_info[2]);
    f.render_widget(widget_carrier_info_other, layout_carrier_info[3]);
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
                layout_carrier_search_list[0].x + 1 + app.carrier_search.cursor_position as u16,
                // Move one line down, from the border to the input line
                layout_carrier_search_list[0].y + 1,
            )
        }
    }
}

// About --------------------------------------------------------------------------------------------------------------------------------------------------------------------

fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame) {
    // data here if needed
    let data_controls_list = vec![
        "Quit: Q, Change Tabs: q and e",
        "Change System/Material List/Prospector: Left and Right arrows",
        "Change Body/Material/Cargo Item selection: Up and Down arrows",
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
                .borders(Borders::TOP | Borders::LEFT)
                .white(),
        );

    let widget_about_version = Paragraph::new(env!("CARGO_PKG_VERSION")).block(
        Block::default()
            .title(" edcas version ")
            .borders(Borders::TOP | Borders::LEFT)
            .white(),
    );

    let widget_about_controls = List::new(data_controls_list).block(
        Block::default()
            .title(" Controls ")
            .borders(Borders::TOP | Borders::LEFT)
            .white(),
    );

    // Render calls
    f.render_widget(widget_about_controls, layout_about[0]);
    f.render_widget(widget_about_github, layout_about[1]);
    f.render_widget(widget_about_version, layout_about[2]);
}
