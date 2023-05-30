use std::{env, fs, thread};
use std::fs::File;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use bus::{Bus, BusReader};
use chrono::Local;
use eframe::App;
use eframe::egui;
use eframe::egui::accesskit::Role::Directory;
use json::JsonValue;
use log::info;

use crate::app::inventory::InventoryState;
use crate::app::State::{About, Explorer, Inventory, Settings};
use crate::egui::Context;

mod about;
mod settings;
mod simple_transaction;
mod explorer;
mod journal_reader;
mod journal_interpreter;
mod inventory;
mod tangle_interpreter;

pub struct EliteRustClient {
    about: about::About,
    explorer: explorer::Explorer,
    state: State,
    journal_log_bus_reader: BusReader<JsonValue>,
    inventory: InventoryState,
    settings: settings::Settings,
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

        println!("Log file: {:?}",path.clone());
        let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()

            .name(path.to_str().unwrap())
            .target_exclusions(&["h2", "hyper", "rustls","iota_wallet","iota_client","reqwest"])
            .level_filter(log::LevelFilter::from_str(settings.log_level.as_str()).unwrap());

        let _logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
            .name(path.to_str().unwrap())
            .target_exclusions(&["h2", "hyper", "rustls"])
            .level_filter(log::LevelFilter::from_str(settings.log_level.as_str()).unwrap());

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
        info!("Starting Tangle Interpreter");
        //Buffer needs to be this large or in development, when the reader timeout is set to 0 the buffer can get full
        let local_settings = settings.clone();
        thread::spawn(move || {
            let mut tangle_interpreter = tangle_interpreter::initialize(tangle_journal_bus_reader, local_settings);
            loop {
                tangle_interpreter.run();
            }
        });
        info!("Done starting threads");

        Self {
            about: about::About::default(),
            explorer: explorer::Explorer::default(),
            state: About,
            journal_log_bus_reader: journal_bus_reader,
            inventory: InventoryState {
                raw: vec![],
                manufactured: vec![],
                encoded: vec![],
                cargo: vec![],
                refinery: vec![],
            },
            settings
        }
    }
}

enum State {
    About,
    Settings,
    Explorer,
    Inventory,
}

impl App for EliteRustClient {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        let Self {
            about, explorer, state, journal_log_bus_reader, inventory,settings
        } = self;

        let mut style: egui::Style = (*ctx.style()).clone();
        for (_text_style, font_id) in style.text_styles.iter_mut() {
            font_id.size = 24.0;
        }
        ctx.set_style(style);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Top panel as menu bar
            egui::menu::bar(ui, |menu_bar| {
                if menu_bar.button("Explorer").clicked() {
                    *state = Explorer;
                }
                if menu_bar.button("Inventory").clicked() {
                    *state = Inventory;
                }
                if menu_bar.button("Settings").clicked() {
                    *state = Settings;
                }
                if menu_bar.button("About").clicked() {
                    *state = About;
                }
            });
        });

        match journal_log_bus_reader.try_recv() {
            Ok(val) => {
                journal_interpreter::interpret_json(val.clone(), explorer,  inventory);
            }
            Err(_) => {}
        }

        egui::CentralPanel::default().show(ctx, |_ui| {
            match self.state {
                About => { about::update(  about, ctx, frame) }
                Settings => { settings.update(ctx,frame) }
                Explorer => { explorer::update(explorer, ctx, frame) }
                Inventory => { inventory.update(ctx,frame)}
            }
        });
        //TODO more efficient way to send updates -> render only if new data comes in?
        //Low prio because performance is okay
        ctx.request_repaint();
    }
}