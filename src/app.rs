use std::{env, fs, thread};
use std::borrow::{Borrow, BorrowMut};
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

use bus::{Bus, BusReader};
use chrono::Local;
use eframe::App;
use eframe::egui;
use eframe::egui::accesskit::Role::Directory;
use iota_wallet::iota_client::crypto::hashes::Digest;
use json::JsonValue;
use log::info;
use crate::app::cargo_reader::CargoReader;

use crate::app::materials::MaterialState;
use crate::app::State::{About, Explorer, MaterialInventory, Mining, News, Settings};
use crate::egui::Context;

mod about;
mod settings;
mod simple_transaction;
mod explorer;
mod journal_reader;
mod journal_interpreter;
mod materials;
mod tangle_interpreter;
mod news;
mod mining;
mod cargo_reader;

pub struct EliteRustClient {
    about: about::About,
    explorer: explorer::Explorer,
    state: State,
    journal_log_bus_reader: BusReader<JsonValue>,
    inventory: MaterialState,
    settings: settings::Settings,
    news: news::News,
    cargo_reader: Arc<Mutex<CargoReader>>,
    mining: mining::Mining,
}

impl Default for EliteRustClient {
    fn default() -> Self {
        let settings = settings::Settings::default();
        let current_dir = env::current_dir().unwrap();
        let logs_dir = current_dir.join("logs");
        let log_filename = format!("{}.log", Local::now().format("%Y-%m-%d-%H-%M"));

        if let Err(err) = fs::create_dir_all(&logs_dir) {
            println!("Error while creating directory: {:?}", err);
        }

        let log_path = logs_dir.join(&log_filename);
        let path = log_path.strip_prefix(&current_dir).unwrap_or(&log_path);

        if let Ok(entries) = fs::read_dir(&logs_dir) {
            let mut log_files: Vec<PathBuf> = entries
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .collect();

            log_files.sort_by(|a, b| b.metadata().unwrap().modified().unwrap().cmp(&a.metadata().unwrap().modified().unwrap()));

            let logs_to_keep = 5;
            if log_files.len() > logs_to_keep {
                for log_file in log_files.into_iter().skip(logs_to_keep) {
                    if let Err(err) = fs::remove_file(log_file) {
                        println!("Error deleting log file: {:?}", err);
                    }
                }
            }
        }

        println!("Log file: {:?}",path.clone());

        let level = log::LevelFilter::from_str(settings.log_level.as_str()).unwrap();

        println!("Log Level: {:?}",level);

        let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
            .name(path.to_str().unwrap())
            .target_exclusions(&["h2", "hyper", "rustls","iota_wallet","iota_client","reqwest","tree_builder"])
            .level_filter(level);

        let _logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
            .name(path.to_str().unwrap())
            .target_exclusions(&["h2", "hyper", "rustls"])
            .level_filter(level);

        let config = fern_logger::LoggerConfig::build()
            .with_output(logger_output_config)
            .finish();
        fern_logger::logger_init(config).unwrap();

        info!("Starting...");
        info!("Current directory: {:?}", env::current_dir().unwrap());

        info!("Starting threads");
        info!("Starting Journal reader");
        let mut journal_bus: Bus<JsonValue> = Bus::new(100);
        let journal_bus_reader = journal_bus.add_rx();
        let tangle_journal_bus_reader = journal_bus.add_rx();
        let directory_path = settings.journal_directory.clone();
        thread::spawn(move || {
            let mut j_reader = journal_reader::initialize(directory_path);
            loop {
                //Sleep needed, because too frequent reads can lead to read the file while being written to it -> exception from json parser because json is not complete
                sleep(Duration::from_millis(100));
                j_reader.run(&mut journal_bus);
            }
        });
        if settings.allow_share_data {
            info!("Starting Tangle Interpreter");
            //Buffer needs to be this large or in development, when the reader timeout is set to 0 the buffer can get full
            let local_settings = settings.clone();
            thread::spawn(move || {
                let mut tangle_interpreter = tangle_interpreter::initialize(tangle_journal_bus_reader, local_settings);
                loop {
                    tangle_interpreter.run();
                }
            });
        }
        info!("Done starting threads");

        let cargo_reader = Arc::new(Mutex::new((cargo_reader::initialize(settings.journal_directory.clone()))));
        let mining = mining::Mining{
            prospectors: Default::default(),
            cargo: cargo_reader.clone(),
        };

        Self {
            news: news::News::default(),
            about: about::About::default(),
            explorer: explorer::Explorer::default(),
            state: News,
            journal_log_bus_reader: journal_bus_reader,
            inventory: MaterialState {
                raw: vec![],
                manufactured: vec![],
                encoded: vec![]
            },
            settings,
            cargo_reader,
            mining,
        }
    }
}

enum State {
    News,
    About,
    Settings,
    Explorer,
    MaterialInventory,
    Mining,
}

impl App for EliteRustClient {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let Self {
            news, about, explorer, state, journal_log_bus_reader, inventory,settings,mining,cargo_reader
        } = self;

        let mut style: egui::Style = (*ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 24.0;
        }
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Top panel as menu bar
            egui::menu::bar(ui, |menu_bar| {
                let news_button = menu_bar.button("News");
                if news_button.clicked() {
                    *state = News;
                }
                let explorer_button =  menu_bar.button("Explorer");
                if explorer_button.clicked() {
                    *state = Explorer;
                }
                let mining_button = menu_bar.button("Mining");
                if mining_button.clicked() {
                    *state = Mining;
                }
                let materials_button = menu_bar.button("Materials");
                if materials_button.clicked() {
                    *state = MaterialInventory;
                }
                let settings_button = menu_bar.button("Settings");
                if settings_button.clicked() {
                    *state = Settings;
                }
                let about_button = menu_bar.button("About");
                if about_button.clicked() {
                    *state = About;
                }

                match state{
                    News => {news_button.highlight();}
                    About => {about_button.highlight();}
                    Settings => {settings_button.highlight();}
                    Explorer => {explorer_button.highlight();}
                    MaterialInventory => {materials_button.highlight();}
                    Mining => {mining_button.highlight();}
                }
            });
        });

        match journal_log_bus_reader.try_recv() {
            Ok(val) => {
                journal_interpreter::interpret_json(val.clone(), explorer,  inventory, mining);
            }
            Err(_) => {}
        }
        {
            cargo_reader.lock().unwrap().run();
        }


        egui::CentralPanel::default().show(ctx, |_ui| {
            match self.state {
                News => { news.update(ctx,frame) }
                About => { about::update(  about, ctx, frame) }
                Settings => { settings.update(ctx,frame) }
                Explorer => { explorer::update(explorer, ctx, frame) }
                MaterialInventory => { inventory.update(ctx, frame)}
                Mining => {mining.update(ctx,frame)}
            }
        });
        //TODO more efficient way to send updates -> render only if new data comes in?
        //Low prio because performance is okay
        ctx.request_repaint();
    }
}