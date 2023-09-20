use std::{env, fs, thread};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use bus::{Bus, BusReader};
use chrono::Local;
use eframe::{App, Frame};
use eframe::egui;
use eframe::egui::{ColorImage, Image, TextStyle, TextureHandle, Vec2};
use eframe::glow::Texture;

use json::JsonValue;
use log::info;
use num_format::Locale::lo;
use crate::app::cargo_reader::CargoReader;

use crate::app::materials::MaterialState;
use crate::app::State::{About, Explorer, MaterialInventory, Mining, News, Settings};
use crate::egui::Context;

mod about;
mod settings;
pub mod explorer;
mod journal_reader;
mod journal_interpreter;
mod materials;
mod tangle_interpreter;
mod news;
mod mining;
mod cargo_reader;

pub struct EliteRustClient {
    pub about: about::About,
    pub explorer: explorer::Explorer,
    pub state: State,
    pub journal_log_bus_reader: BusReader<JsonValue>,
    pub materials: MaterialState,
    pub settings: settings::Settings,
    pub news: news::News,
    pub cargo_reader: Arc<Mutex<CargoReader>>,
    pub mining: mining::Mining,
    pub timestamp: String,
}

impl EliteRustClient {
    pub fn update_values(&mut self) {
        match self.journal_log_bus_reader.try_recv() {
            Ok(json) => {
                self.timestamp = json["timestamp"].to_string();
                journal_interpreter::interpret_json(json.clone(), &mut self.explorer, &mut self.materials, &mut self.mining, Arc::new(self.settings.clone()));
            }
            Err(_) => {}
        }
        {
            self.cargo_reader.lock().unwrap().run();
        }
    }
}
impl Default for EliteRustClient {
    fn default() -> Self {
        let settings = settings::Settings::default();
        let settings_pointer = Arc::new(settings.clone());
        initialize_logger(settings_pointer.clone());

        info!("Starting...");
        info!("Current directory: {:?}", env::current_dir().unwrap());
        info!("Reading materials");

        let materials = MaterialState::default();

        info!("Starting threads");
        info!("Starting Journal reader");
        let mut journal_bus: Bus<JsonValue> = Bus::new(100);
        let journal_bus_reader = journal_bus.add_rx();
        let tangle_journal_bus_reader = journal_bus.add_rx();
        let settings_pointer_clone = settings_pointer.clone();
        thread::spawn(move || {
            let mut j_reader = journal_reader::initialize(settings_pointer_clone);
            loop {
                //Sleep needed, because too frequent reads can lead to read the file while being written to it -> exception from json parser because json is not complete
                sleep(Duration::from_millis(100));
                j_reader.run(&mut journal_bus);
            }
        });
        let settings_pointer_clone = settings_pointer.clone();
        info!("Allow to share data over edcas: {}", settings_pointer.iota_settings.allow_share_data);
        if settings_pointer.iota_settings.allow_share_data {
            info!("Starting Tangle Interpreter");
            //Buffer needs to be this large or in development, when the reader timeout is set to 0 the buffer can get full
            let settings_pointer = settings_pointer_clone;
            thread::spawn(move || {
                let mut tangle_interpreter = tangle_interpreter::initialize(tangle_journal_bus_reader, settings_pointer);
                loop {
                    tangle_interpreter.run();
                }
            });
        }
        info!("Done starting threads");

        let cargo_reader = Arc::new(Mutex::new(cargo_reader::initialize(settings_pointer.clone())));
        let mining = mining::Mining{
            prospectors: Default::default(),
            cargo: cargo_reader.clone(),
        };

        Self {
            news: news::News::default(),
            about: about::About::default(),
            explorer: explorer::Explorer{
                systems: vec![],
                index: 0,
                body_list_index: None,
                settings: settings_pointer.clone(),
            },
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

pub enum State {
    News,
    About,
    Settings,
    Explorer,
    MaterialInventory,
    Mining,
}

impl App for EliteRustClient {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        if !self.settings.appearance_settings.applied {

            let mut style: egui::Style = (*ctx.style()).clone();
            for (text_style, font_id) in style.text_styles.iter_mut() {
                match text_style {
                    TextStyle::Small => {
                        if  self.settings.appearance_settings.font_id.size > 4.0 {
                            font_id.size = self.settings.appearance_settings.font_id.size - 4.0;
                        }else {
                            font_id.size = self.settings.appearance_settings.font_id.size;
                        }

                    }
                    TextStyle::Heading => {
                        font_id.size = self.settings.appearance_settings.font_id.size + 4.0;
                    }
                    _ => {
                        font_id.size = self.settings.appearance_settings.font_id.size;
                        font_id.family = self.settings.appearance_settings.font_id.family.clone();
                    }
                }
            }
            ctx.set_style(style);
            self.settings.appearance_settings.font_size = self.settings.appearance_settings.font_id.size;
            self.settings.appearance_settings.font_style = self.settings.appearance_settings.font_id.family.to_string();
            self.settings.appearance_settings.applied = true;
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Top panel as menu bar
            egui::menu::bar(ui, |menu_bar| {
                let news_button = menu_bar.button("News");
                if news_button.clicked() {
                    self.state = News;
                }
                let explorer_button =  menu_bar.button("Explorer");
                if explorer_button.clicked() {
                    self.state = Explorer;
                }
                let mining_button = menu_bar.button("Mining");
                if mining_button.clicked() {
                    self.state = Mining;
                }
                let materials_button = menu_bar.button("Materials");
                if materials_button.clicked() {
                    self.state = MaterialInventory;
                }
                let settings_button = menu_bar.button("Settings");
                if settings_button.clicked() {
                    self.state = Settings;
                }
                let about_button = menu_bar.button("About");
                if about_button.clicked() {
                    self.state = About;
                }

                menu_bar.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label(self.timestamp.as_str());
                });
                match self.state{
                    News => {news_button.highlight();}
                    About => {about_button.highlight();}
                    Settings => {settings_button.highlight();}
                    Explorer => {explorer_button.highlight();}
                    MaterialInventory => {materials_button.highlight();}
                    Mining => {mining_button.highlight();}
                }
            });
        });

        self.update_values();

        egui::CentralPanel::default().show(ctx, |_ui| {
            match self.state {
                News => { self.news.update(ctx,frame) }
                About => { self.about.update(ctx, frame) }
                Settings => { self.settings.update(ctx,frame) }
                Explorer => { self.explorer.update(ctx,frame) }
                MaterialInventory => { self.materials.update(ctx, frame) }
                Mining => { self.mining.update(ctx,frame) }
            }
        });
        //TODO more efficient way to send updates -> render only if new data comes in?
        //Low prio because performance is okay
        ctx.request_repaint();
    }
}

fn initialize_logger(settings: Arc<settings::Settings>) {
    let mut log_directory = env::current_dir().unwrap().join("logs");
    if std::path::Path::new("/tmp/").exists() {
        log_directory = std::path::Path::new("/tmp/edcas-client/").to_path_buf();
    }

    if let Err(err) = fs::create_dir_all(&log_directory) {
        println!("Error while creating log directory: {:?}", err);
    }

    let log_filename = format!("{}.log", Local::now().format("%Y-%m-%d-%H-%M"));


    let log_path = log_directory.join(&log_filename);
    //let log_path = log_file_path_buf.strip_prefix(&log_directory).unwrap_or(&log_file_path_buf);

    if let Ok(entries) = fs::read_dir(&log_directory) {
        let mut log_files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        log_files.sort_by(|a, b| b.metadata().unwrap().modified().unwrap().cmp(&a.metadata().unwrap().modified().unwrap()));

        let logs_to_keep = 5;
        if log_files.len() > logs_to_keep {
            for log_file in log_files.into_iter().skip(logs_to_keep) {
                //println!("Removing old log file: {:?}",&log_file);
                if let Err(err) = fs::remove_file(log_file) {
                    println!("Error deleting old log file: {:?}", err);
                }
            }
        }
    }

    println!("Log file: {:?}", log_path.clone());

    let level = log::LevelFilter::from_str(settings.log_level.as_str()).unwrap();

    println!("New log Level: {:?}",level);

    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name(log_path.to_str().unwrap())
        .target_exclusions(&["h2", "hyper", "rustls","iota_wallet","iota_client","reqwest","tree_builder","html5ever"])
        .level_filter(level);

    let _logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name(log_path.to_str().unwrap())
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(level);

    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();
}