use std::{error::Error, io};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use crate::app::EliteRustClient;

struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> App<'a> {
    fn new() -> App<'a> {
        App {
            titles: vec!["Default", "Explorer", "Mining", "Materials", "About"],
            index: 0,
        }
    }

    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
        }
    }
}

pub fn draw_tui(client: Box<EliteRustClient>) -> Result<(), Box<dyn Error>> {
    // aber die main in main.rs returns kein Result??
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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
        DisableMouseCapture
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
    mut client: Box<EliteRustClient>,
) -> io::Result<()> {
    loop {
        client.update_values();

        terminal.draw(|f| ui(f, &app, &client))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Right | KeyCode::Char('e') => app.next(),
                    KeyCode::Left | KeyCode::Char('q') => app.previous(),
                    _ => {}
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App, client: &Box<EliteRustClient>) {
    let size = f.size();
    //definition of general layout
    let chunks = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(size);

    //
    let titles: Vec<_> = app
        .titles
        .iter()
        .map(|title| {
            let (first, rest) = title.split_at(1);
            Line::from(vec![first.yellow(), rest.green()])
        })
        .collect();
    //render tabs
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("Tabs"))
        .select(app.index)
        .style(Style::default().cyan().on_gray())
        .highlight_style(Style::default().bold().on_black());
    f.render_widget(tabs, chunks[0]);

    //render tab contents
    match app.index {
        0 => tab_default(chunks[1], f, &client),
        1 => tab_explorer(chunks[1], f, &client),
        _ => unreachable!(),
    };
    // f.render_widget(inner, chunks[1]);
}

// Tabs

fn tab_default(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &Box<EliteRustClient>,
) {
    let widget_default = Paragraph::new("default text here").block(
        Block::default()
            .title("Default")
            .borders(Borders::ALL)
            .white()
            .on_black(),
    );

    f.render_widget(widget_default, chunk);
}

fn tab_explorer(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: &Box<EliteRustClient>,
) {
    //general layout
    let layout_explorer = ratatui::prelude::Layout::default()
        .direction(ratatui::prelude::Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage(20),
            Constraint::Fill(1),
            Constraint::Percentage(20),
        ])
        .split(chunk);

    //widget for body list
    let widget_signal_list = List::new(
        client.explorer.systems[1]
            .signal_list
            .iter()
            .map(|signal| signal.clone().name)
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Signals")
            .borders(Borders::ALL)
            .white()
            .on_black(),
    );

    f.render_widget(widget_signal_list, layout_explorer[0]);

    //widget for body info
    let widget_body_list = List::new(
        client.explorer.systems[0]
            .body_list
            .iter()
            .map(|body| body.get_name())
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Body List")
            .borders(Borders::TOP)
            .borders(Borders::BOTTOM)
            .white()
            .on_black(),
    );

    f.render_widget(widget_body_list, layout_explorer[1]);

    let widget_body_signals_list = List::new(
        client.explorer.systems[0].body_list[0]
            .get_signals()
            .iter()
            .map(|body_signal| body_signal.clone().r#type)
            .collect::<Vec<_>>(),
    )
    .block(
        Block::default()
            .title("Body Signals")
            .borders(Borders::ALL)
            .white()
            .on_black(),
    );

    f.render_widget(widget_body_signals_list, layout_explorer[2]);
}

fn tab_mining(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame, client: Box<EliteRustClient>) {}

fn tab_materials(
    chunk: ratatui::layout::Rect,
    f: &mut ratatui::Frame,
    client: Box<EliteRustClient>,
) {
}

fn tab_about(chunk: ratatui::layout::Rect, f: &mut ratatui::Frame, client: Box<EliteRustClient>) {
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
            .black()
            .on_white(),
    );
    f.render_widget(widget_about_github, layout_about[0]);

    let widget_about_version = Paragraph::new("version").block(
        Block::default()
            .title("edcas version")
            .borders(Borders::ALL)
            .black()
            .on_white(),
    );
    f.render_widget(widget_about_version, layout_about[1]);
}
