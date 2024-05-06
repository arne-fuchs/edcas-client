use crate::edcas::backend::evm::request_handler::EvmRequest;
use crate::edcas::EliteRustClient;
use crate::tui::{App, InputMode};
use ratatui::{prelude::*, style::Stylize, widgets::*};
use std::cmp::Ordering;

pub fn tab_dockables(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &mut EliteRustClient,
    app: &mut App,
) {
    app.dockable_list_state
        .select(Some(app.dockable_list_index));

    let dataset_carriers_list_selected: Vec<_>;
    let dataset_stations_list_selected: Vec<_>;

    let mut data_dockable_list_selected: Vec<String> = vec!["no data".to_string()];
    let mut data_dockable_info_location = "no data".to_string();
    let mut data_carrier_info_destination = "no data".to_string();
    let mut data_dockable_info_modules: Vec<String> = vec!["no data".to_string()];
    let mut data_dockable_info_other = "no data".to_string();
    let mut data_station_info = vec![Row::new(vec!["no data".to_string()])];

    //common layout definitions
    let layout_carrier = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(46), Constraint::Fill(1)])
        .split(chunk);

    let layout_dockable_search_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(layout_carrier[0]);

    //tab dependent data, layout and widgets definitions
    match app.dockable_mode {
        super::DockableMode::Carriers => {
            if !client.carrier.carriers.is_empty() {
                dataset_carriers_list_selected = client
                    .carrier
                    .carriers
                    .iter()
                    .filter(|f| {
                        f.name
                            .to_lowercase()
                            .contains(&app.dockable_search.input.to_lowercase())
                            || f.callsign
                                .to_lowercase()
                                .contains(&app.dockable_search.input.to_lowercase())
                    })
                    .collect::<Vec<_>>();

                /*
                for carrier in &client.carrier.carriers {
                    if carrier
                        .name
                        .to_lowercase()
                        .contains(&app.dockable_search.input.to_lowercase())
                        || carrier
                            .callsign
                            .to_lowercase()
                            .contains(&app.dockable_search.input.to_lowercase())
                    {
                        dataset_carriers_list_selected.push(carrier);
                    }
                }
                */

                data_dockable_list_selected = dataset_carriers_list_selected
                    .iter()
                    .map(|carrier| {
                        [carrier.name.to_string(), carrier.callsign.to_string()].join(" - ")
                    })
                    .collect::<Vec<String>>();

                if !data_dockable_list_selected.is_empty() {
                    match app
                .dockable_list_index
                .cmp(&data_dockable_list_selected.len())  //thats fucked up but it works
            {
                Ordering::Greater => app.dockable_list_index = data_dockable_list_selected.len() - 1,
                Ordering::Equal => app.dockable_list_index = 0,
                _ => {}
            }

                    data_dockable_info_location = [
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .current_system
                            .to_string(),
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .current_body
                            .to_string(),
                    ]
                    .join(" - ");

                    data_carrier_info_destination = [
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .next_system
                            .to_string(),
                        " - ".to_string(),
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .next_body
                            .to_string(),
                        "\n".to_string(),
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .departure
                            .to_string(),
                    ]
                    .join("");

                    data_dockable_info_modules = dataset_carriers_list_selected
                        [app.dockable_list_index]
                        .services
                        .split(',')
                        .map(|f| f.to_string())
                        .collect::<Vec<String>>();

                    data_dockable_info_other = [
                        "Allow notorious:".to_string(),
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .allow_notorious
                            .to_string(),
                        "\nDocking Access:".to_string(),
                        dataset_carriers_list_selected[app.dockable_list_index]
                            .docking_access
                            .to_string(),
                    ]
                    .join(" ");
                }
            } else {
                app.dockable_list_index = 0;
            }

            let layout_dockable_info = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),                                           // Location
                    Constraint::Length(4),                                           // Destination
                    Constraint::Length(data_dockable_info_modules.len() as u16 + 2), // Modules
                    Constraint::Fill(1),                                             // Other
                ])
                .split(layout_carrier[1]);

            let widget_dockable_info_location = Paragraph::new(data_dockable_info_location)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(" Location ")
                        .bold()
                        .borders(Borders::TOP | Borders::LEFT),
                );
            let widget_dockable_info_destination = Paragraph::new(data_carrier_info_destination)
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title(" Jump Destination ")
                        .bold()
                        .borders(Borders::TOP | Borders::LEFT),
                );
            let widget_dockable_info_modules = List::new(data_dockable_info_modules).block(
                Block::default()
                    .title(" Available Services ")
                    .bold()
                    .borders(Borders::TOP | Borders::LEFT),
            );
            let widget_dockable_info_other = Paragraph::new(data_dockable_info_other).block(
                Block::default()
                    .title(" Other ")
                    .bold()
                    .borders(Borders::TOP | Borders::LEFT),
            );

            f.render_widget(widget_dockable_info_location, layout_dockable_info[0]);
            f.render_widget(widget_dockable_info_destination, layout_dockable_info[1]);
            f.render_widget(widget_dockable_info_modules, layout_dockable_info[2]);
            f.render_widget(widget_dockable_info_other, layout_dockable_info[3]);
        }

        super::DockableMode::Stations => {
            if !client.station.stations.is_empty() {
                dataset_stations_list_selected = client
                    .station
                    .stations
                    .iter()
                    .filter(|f| {
                        f.name
                            .to_lowercase()
                            .contains(&app.dockable_search.input.to_lowercase())
                    })
                    .collect::<Vec<_>>();

                /*
                for station in &client.station.stations {
                    if station
                        .name
                        .to_lowercase()
                        .contains(&app.dockable_search.input.to_lowercase())
                    {
                        dataset_stations_list_selected.push(station);
                    }
                }
                */

                data_dockable_list_selected = dataset_stations_list_selected
                    .iter()
                    .map(|station| station.name.to_string())
                    .collect::<Vec<String>>();

                if !data_dockable_list_selected.is_empty() {
                    match app
                .dockable_list_index
                .cmp(&data_dockable_list_selected.len())  //thats fucked up but it works
            {
                Ordering::Greater => app.dockable_list_index = data_dockable_list_selected.len() - 1,
                Ordering::Equal => app.dockable_list_index = 0,
                _ => {}
            }

                    match &dataset_stations_list_selected[app.dockable_list_index].meta_data {
                        None => {
                            if !dataset_stations_list_selected[app.dockable_list_index]
                                .requested_meta_data
                            {
                                if let Err(err) = client.station.evm_request_writer.send(
                                    EvmRequest::StationMetaData(
                                        dataset_stations_list_selected[app.dockable_list_index]
                                            .market_id,
                                    ),
                                ) {
                                    data_dockable_info_other = [
                                        "Error sending Station MetaData Request:".to_string(),
                                        err.to_string(),
                                    ]
                                    .join(" ");
                                }

                                let market_id_index = client
                                    .station
                                    .stations
                                    .iter()
                                    .position(|station| {
                                        station.market_id
                                            == dataset_stations_list_selected
                                                [app.dockable_list_index]
                                                .market_id
                                    })
                                    .unwrap();
                                client.station.stations[market_id_index].requested_meta_data = true;
                            } else {
                                data_station_info = vec![Row::new(vec!["Fetching".to_string()])]
                            }
                        }
                        Some(station_metadata) => {
                            data_station_info = vec![
                                Row::new(vec![
                                    "Last update".to_string(),
                                    station_metadata.timestamp.to_string(),
                                ]),
                                Row::new(vec![
                                    "Location".to_string(),
                                    station_metadata.system_name.to_string(),
                                ]),
                                Row::new(vec!["Distance".to_string(), {
                                    let mut distance =
                                        station_metadata.distance.decimal.to_string();
                                    distance.insert(
                                        station_metadata.distance.floating_point as usize - 1,
                                        '.',
                                    );
                                    distance
                                }]),
                                Row::new(vec![
                                    "Economy".to_string(),
                                    station_metadata.economy.split('_').collect::<Vec<_>>()[1]
                                        .to_string(),
                                ]),
                                Row::new(vec![
                                    "Government".to_string(),
                                    station_metadata.government.split('_').collect::<Vec<_>>()[1]
                                        .to_string(),
                                ]),
                                //Row::new(vec![]),
                            ];
                            data_dockable_info_modules = station_metadata
                                .services
                                .split(',')
                                .map(|f| f.to_string())
                                .collect::<Vec<String>>();
                            data_dockable_info_other = station_metadata.landingpads.to_string();
                        }
                    }
                }
            } else {
                app.dockable_list_index = 0;
            }

            let layout_dockable_info = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(data_station_info.len() as u16 + 2),
                    Constraint::Length(data_dockable_info_modules.len() as u16 + 2), // services
                    Constraint::Fill(1),                                             // other
                ])
                .split(layout_carrier[1]);

            let widget_dockable_info_table = Table::new(
                data_station_info,
                [Constraint::Length(14), Constraint::Length(20)],
            )
            .block(
                Block::default()
                    .title(" Info ")
                    .bold()
                    .borders(Borders::TOP | Borders::LEFT),
            );

            let widget_dockable_info_modules = List::new(data_dockable_info_modules).block(
                Block::default()
                    .title(" Available Services ")
                    .bold()
                    .borders(Borders::TOP | Borders::LEFT),
            );
            let widget_dockable_info_other = Paragraph::new(data_dockable_info_other).block(
                Block::default()
                    .title(" Landing Pads ")
                    .bold()
                    .borders(Borders::TOP | Borders::LEFT),
            );

            f.render_widget(widget_dockable_info_table, layout_dockable_info[0]);
            f.render_widget(widget_dockable_info_modules, layout_dockable_info[1]);
            f.render_widget(widget_dockable_info_other, layout_dockable_info[2]);
        }
    }

    //common widget definitions
    let widget_dockable_search = Paragraph::new(app.dockable_search.input.clone()).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT)
            .white()
            .title(" Search "),
    );

    let widget_dockable_list = List::new(data_dockable_list_selected)
        .block(
            Block::default()
                .title(
                    [
                        " ◁ Known ".to_string(),
                        app.dockable_mode.non_display_to_string(),
                        " ▷ ".to_string(),
                    ]
                    .join(""),
                )
                .borders(Borders::TOP | Borders::LEFT)
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    //common render calls
    f.render_widget(widget_dockable_search, layout_dockable_search_list[0]);
    f.render_stateful_widget(
        widget_dockable_list,
        layout_dockable_search_list[1],
        &mut app.dockable_list_state,
    );

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
                layout_dockable_search_list[0].x + 1 + app.dockable_search.cursor_position as u16,
                // Move one line down, from the border to the input line
                layout_dockable_search_list[0].y + 1,
            )
        }
    }
}
