use crate::edcas::{materials::Material, EliteRustClient};
use crate::tui::App;
use crate::tui::InputMode;
use ratatui::{prelude::*, style::Stylize, widgets::*};

pub fn tab_materials(
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
        .constraints([Constraint::Length(46), Constraint::Fill(1)])
        .split(chunk);

    let layout_materials_search_list = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Fill(1)])
        .split(layout_materials[0]);

    let layout_materials_list = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(7)])
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
