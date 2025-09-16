mod edcas_error;
mod eddn_adapter;
mod interpreter;
mod listener;
mod parser;

mod faction;
mod star_system;

use log::{debug, error, info, warn};

pub fn run_listener() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            initialize_logging("eddn_listener");
            listener::run().await;
        });
}
pub fn run_parser() {
    initialize_logging("eddn_parser");
    parser::run();
}

fn initialize_logging(module: &str) {
    std::fs::create_dir_all("log").expect("Failed to create log directory");
    let mut dispatch = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}]{}",
                chrono::Utc::now().to_rfc3339(),
                record.level(),
                message
            ))
        })
        //.chain(std::io::stdout())
        .level(log::LevelFilter::Error)
        .level(log::LevelFilter::Warn)
        .level(log::LevelFilter::Info)
        .level_for("tokio_postgres", log::LevelFilter::Info)
        .chain(
            fern::log_file(format!(
                "log/{}-{}.log",
                module,
                chrono::Utc::now().to_rfc3339()
            ))
            .unwrap(),
        );

    #[cfg(debug_assertions)]
    {
        dispatch = dispatch.level(log::LevelFilter::Debug);
    }
    dispatch.apply().expect("Unable to initialize logger");
    debug!("DEBUG: Active");
    info!("INFO: Active");
    warn!("WARN: Acitve");
    error!("ERROR: Active");
}
