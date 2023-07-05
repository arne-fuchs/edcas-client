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
use num_format::Locale::en;
use serde_json::json;
use crate::app::cargo_reader::CargoReader;

use crate::app::materials::{Material, MaterialState};
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
    materials: MaterialState,
    settings: settings::Settings,
    news: news::News,
    cargo_reader: Arc<Mutex<CargoReader>>,
    mining: mining::Mining,
    timestamp: String,
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
        info!("Reading materials");

        let mut materials = MaterialState::default();
        let materials_content = fs::read_to_string("materials.json").unwrap();
        let materials_json = json::parse(materials_content.as_str()).unwrap();

        let encoded_array = &materials_json["encoded"];
        for i in 0..encoded_array.len(){
            let encoded = &encoded_array[i];

            let mut locations: Vec<String> = vec![];
            let locations_array = &encoded["locations"];
            for j in 0..locations_array.len(){
                locations.push(locations_array[j].to_string())
            }

            let mut sources: Vec<String> = vec![];
            let sources_array = &encoded["sources"];
            for j in 0..sources_array.len(){
                sources.push(sources_array[j].to_string())
            }

            let mut engineering: Vec<String> = vec![];
            let engineering_array = &encoded["engineering"];
            for j in 0..engineering_array.len(){
                engineering.push(engineering_array[j].to_string())
            }

            let mut synthesis: Vec<String> = vec![];
            let synthesis_array = &encoded["synthesis"];
            for j in 0..synthesis_array.len(){
                synthesis.push(synthesis_array[j].to_string())
            }


            materials.encoded.insert(
                encoded["name"].to_string(),
                Material{
                    name: encoded["name"].to_string(),
                    name_localised: encoded["name_localised"].to_string(),
                    grade: encoded["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: encoded["maximum"].as_u64().unwrap(),
                    category: encoded["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: encoded["description"].to_string(),
                }
            );
        }

        let manufactured_array = &materials_json["manufactured"];
        for i in 0..manufactured_array.len(){
            let manufactured = &manufactured_array[i];

            let mut locations: Vec<String> = vec![];
            let locations_array = &manufactured["locations"];
            for j in 0..locations_array.len(){
                locations.push(locations_array[j].to_string())
            }

            let mut sources: Vec<String> = vec![];
            let sources_array = &manufactured["sources"];
            for j in 0..sources_array.len(){
                sources.push(sources_array[j].to_string())
            }

            let mut engineering: Vec<String> = vec![];
            let engineering_array = &manufactured["engineering"];
            for j in 0..engineering_array.len(){
                engineering.push(engineering_array[j].to_string())
            }

            let mut synthesis: Vec<String> = vec![];
            let synthesis_array = &manufactured["synthesis"];
            for j in 0..synthesis_array.len(){
                synthesis.push(synthesis_array[j].to_string())
            }


            materials.manufactured.insert(
                manufactured["name"].to_string(),
                Material{
                    name: manufactured["name"].to_string(),
                    name_localised: manufactured["name_localised"].to_string(),
                    grade: manufactured["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: manufactured["maximum"].as_u64().unwrap(),
                    category: manufactured["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: manufactured["description"].to_string(),
                }
            );
        }

        let raw_array = &materials_json["raw"];
        for i in 0..raw_array.len(){
            let raw = &raw_array[i];

            let mut locations: Vec<String> = vec![];
            let locations_array = &raw["locations"];
            for j in 0..locations_array.len(){
                locations.push(locations_array[j].to_string())
            }

            let mut sources: Vec<String> = vec![];
            let sources_array = &raw["sources"];
            for j in 0..sources_array.len(){
                sources.push(sources_array[j].to_string())
            }

            let mut engineering: Vec<String> = vec![];
            let engineering_array = &raw["engineering"];
            for j in 0..engineering_array.len(){
                engineering.push(engineering_array[j].to_string())
            }

            let mut synthesis: Vec<String> = vec![];
            let synthesis_array = &raw["synthesis"];
            for j in 0..synthesis_array.len(){
                synthesis.push(synthesis_array[j].to_string())
            }


            materials.raw.insert(
                raw["name"].to_string(),
                Material{
                    name: raw["name"].to_string(),
                    name_localised: raw["name_localised"].to_string(),
                    grade: raw["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: raw["maximum"].as_u64().unwrap(),
                    category: raw["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: raw["description"].to_string(),
                }
            );
        }


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
            materials,
            settings,
            cargo_reader,
            mining,
            timestamp: String::from(""),
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
            news, about, explorer, state, journal_log_bus_reader, materials: inventory,settings,mining,cargo_reader,timestamp
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

                menu_bar.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label(timestamp.as_str());
                });
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
            Ok(json) => {
                self.timestamp = json["timestamp"].to_string();
                journal_interpreter::interpret_json(json.clone(), explorer,  inventory, mining);
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