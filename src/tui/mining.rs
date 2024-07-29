use crate::edcas::{mining::MiningMaterial, EliteRustClient};
use crate::tui::App;
use core::f64;
use ratatui::{prelude::*, style::Stylize, widgets::*};

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
        ["┌".to_string(), mining_content.to_owned(), {
            let mut line = "─".to_string();
            for _i in 0..(35 - mining_content.len()) {
                line.push('─');
            }
            line
        }]
        .join(" "),
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

pub fn tab_mining(
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
        //*app.prospector_list_state.offset_mut() = app.prospector_index;
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
