use ratatui::{prelude::*, style::Stylize, widgets::*};

pub fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame) {
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
