use crate::app::EliteRustClient;
use crate::tui::{App, InputMode};
use ratatui::{prelude::*, style::Stylize, widgets::*};
use std::cmp::Ordering;

pub fn tab_dockables(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &EliteRustClient,
    app: &mut App,
) {
    app.dockable_list_state
        .select(Some(app.dockable_list_index));

    let mut dataset_dockable_list_selected: Vec<_> = vec![];

    let mut data_dockable_list_selected: Vec<String> = vec!["no data".to_string()];
    let mut data_dockable_info_location = "no data".to_string();
    let mut data_dockable_info_destination = "no data".to_string();
    let mut data_dockable_info_modules: Vec<String> = vec!["no data".to_string()];
    let mut data_dockable_info_other = "no data".to_string();

    /*
    fn filter_by_input(&mut self, list: Vec<String>) -> Vec<String> {
        let mut list_selected: Vec<String> = vec![];
        for value in list {
            if value.to_lowercase().contains(&self.input.to_lowercase()) {
                list_selected.push(value);
            }
        }
        list_selected
    }*/

    if !client.carrier.carriers.is_empty() {
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
                dataset_dockable_list_selected.push(carrier);
            }
        }

        data_dockable_list_selected = dataset_dockable_list_selected
            .iter()
            .map(|carrier| [carrier.name.to_string(), carrier.callsign.to_string()].join(" - "))
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
                dataset_dockable_list_selected[app.dockable_list_index]
                    .current_system
                    .to_string(),
                dataset_dockable_list_selected[app.dockable_list_index]
                    .current_body
                    .to_string(),
            ]
            .join(" - ");

            data_dockable_info_destination = [
                dataset_dockable_list_selected[app.dockable_list_index]
                    .next_system
                    .to_string(),
                " - ".to_string(),
                dataset_dockable_list_selected[app.dockable_list_index]
                    .next_body
                    .to_string(),
                "\n".to_string(),
                dataset_dockable_list_selected[app.dockable_list_index]
                    .departure
                    .to_string(),
            ]
            .join("");

            data_dockable_info_modules = dataset_dockable_list_selected[app.dockable_list_index]
                .services
                .split(',')
                .map(|f| f.to_string())
                .collect::<Vec<String>>();

            data_dockable_info_other = [
                "Allow notorious:".to_string(),
                dataset_dockable_list_selected[app.dockable_list_index]
                    .allow_notorious
                    .to_string(),
                "\nDocking Access:".to_string(),
                dataset_dockable_list_selected[app.dockable_list_index]
                    .docking_access
                    .to_string(),
            ]
            .join(" ");
        }
    }

    let layout_carrier = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(46), Constraint::Fill(1)])
        .split(chunk);

    let layout_dockable_search_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(layout_carrier[0]);

    let layout_dockable_info = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                                           // Location
            Constraint::Length(4),                                           // Destination
            Constraint::Length(data_dockable_info_modules.len() as u16 + 2), // Modules
            Constraint::Fill(1),                                             // Other
        ])
        .split(layout_carrier[1]);

    // Widget definitions
    let widget_dockable_search = Paragraph::new(app.dockable_search.input.clone()).block(
        Block::default()
            .borders(Borders::TOP | Borders::LEFT)
            .white()
            .title(" Search "),
    );

    let widget_dockable_list = List::new(data_dockable_list_selected)
        .block(
            Block::default()
                .title(" Known Carriers ")
                .borders(Borders::TOP | Borders::LEFT)
                .white(),
        )
        .highlight_style(Style::default().bold().white().on_dark_gray());

    let widget_dockable_info_location = Paragraph::new(data_dockable_info_location)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .title(" Location ")
                .bold()
                .borders(Borders::TOP | Borders::LEFT),
        );
    let widget_dockable_info_destination = Paragraph::new(data_dockable_info_destination)
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

    // Render calls
    f.render_widget(widget_dockable_search, layout_dockable_search_list[0]);
    f.render_stateful_widget(
        widget_dockable_list,
        layout_dockable_search_list[1],
        &mut app.dockable_list_state,
    );

    f.render_widget(widget_dockable_info_location, layout_dockable_info[0]);
    f.render_widget(widget_dockable_info_destination, layout_dockable_info[1]);
    f.render_widget(widget_dockable_info_modules, layout_dockable_info[2]);
    f.render_widget(widget_dockable_info_other, layout_dockable_info[3]);
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
