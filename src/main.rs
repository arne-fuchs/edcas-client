use std::io::stdout;
use std::process::exit;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use tracing::{error, info};
use tracing_appender::non_blocking::WorkerGuard;

mod api_client;
mod app;
mod cli;
mod eddn;
mod engineering_data;
mod event_shim;
mod journal_reader;
mod my_carriers;
mod pins;
mod settings;
mod todo;
mod views;

use app::App;

fn main() -> Result<()> {
    let _file_guard = init_file_logging();

    info!("EDCAS client starting");

    let args: Vec<String> = std::env::args().collect();
    info!("CLI arguments: {:?}", args);

    for arg in &args {
        match arg.as_str() {
            "--help" => {
                println!("EDCAS client — Elite Dangerous Commander Assistant System");
                println!();
                println!("Options:");
                println!("  --upload-logs   Upload all journal log files to the configured API server and exit.");
                println!("  --help          Show this help message.");
                println!();
                println!("Run `edcas-server` (separate binary) to start the EDDN listener and API.");
                exit(0);
            }
            "--upload-logs" => {
                cli::upload_logs();
                exit(0);
            }
            _ => {}
        }
    }

    enable_raw_mode()?;
    info!("Terminal raw mode enabled");
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        error!("Application error: {:?}", err);
        eprintln!("Error: {:?}", err);
        exit(1);
    }

    info!("Application exited cleanly");
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    info!("Entering main application loop");
    loop {
        app.poll_journal_updates();
        app.poll_search_results();
        app.settings_view.poll_bulk_upload();

        terminal.draw(|frame| app.render(frame))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    info!("Key pressed: {:?}", key);
                    app.handle_key(&key);
                    if app.should_quit {
                        info!("Quit requested");
                        return Ok(());
                    }
                }
            }
        }
    }
}

fn init_file_logging() -> WorkerGuard {
    let file = std::fs::File::create("log.log").expect("Failed to create log file");
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            tracing_subscriber::EnvFilter::new("warn,edcas_client=debug")
        });

    let subscriber = tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_env_filter(filter)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    guard
}
