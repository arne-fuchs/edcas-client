use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, thread};

use bus::{Bus, BusReader};
use chrono::Local;

use crate::edcas::backend::cargo_reader;
use crate::edcas::backend::cargo_reader::CargoReader;
use crate::edcas::backend::evm::request_handler;
use crate::edcas::backend::evm::request_handler::{EvmRequest, EvmUpdate};
use crate::edcas::backend::journal_reader;
use crate::edcas::carrier::CarrierState;

use crate::edcas::backend::evm::edcas_contract::StationIdentity;
use crate::edcas::explorer::system::PlanetSignal;
use json::JsonValue;
use log::info;
use num_format::Locale::fo;

use crate::edcas::materials::MaterialState;
use crate::edcas::station::{Station, StationState};
use crate::edcas::State::News;

pub mod explorer;

pub(crate) mod about;
pub(crate) mod backend;
pub(crate) mod carrier;
pub mod materials;
pub mod mining;
pub(crate) mod news;
pub(crate) mod settings;
pub(crate) mod station;

pub struct EliteRustClient {
    pub about: about::About,
    pub explorer: explorer::Explorer,
    pub station: StationState,
    pub carrier: CarrierState,
    pub state: State,
    pub materials: MaterialState,
    pub settings: settings::Settings,
    pub news: news::News,
    pub mining: mining::Mining,
    pub cargo_reader: Arc<Mutex<CargoReader>>,
    pub journal_log_bus_reader: BusReader<JsonValue>,
    pub evm_update_reader: BusReader<EvmUpdate>,
    pub evm_request_writer: Sender<EvmRequest>,
    pub timestamp: String,
}

impl EliteRustClient {
    pub fn update_values(&mut self) {
        if let Ok(json) = self.journal_log_bus_reader.try_recv() {
            self.timestamp = json["timestamp"].to_string();
            backend::journal_interpreter::interpret_json(
                json.clone(),
                &self.evm_request_writer,
                &mut self.explorer,
                &mut self.materials,
                &mut self.mining,
                Arc::new(self.settings.clone()),
            );
        }
        {
            self.cargo_reader.lock().unwrap().run();
        }
        {
            if let Ok(update) = self.evm_update_reader.try_recv() {
                match update {
                    EvmUpdate::CarrierList(carriers) => {
                        self.carrier.carriers = carriers;
                    }
                    EvmUpdate::StationList(stations) => {
                        for station_identity in stations {
                            //List is sorted by name -> If name exceeds alphabetic order, it is not in the list and can be added
                            if !self.is_station_registered(&station_identity) {
                                self.station.stations.push(Station {
                                    market_id: station_identity.market_id,
                                    name: station_identity.name,
                                    _type: station_identity.type_,
                                    requested_meta_data: false,
                                    requested_market: false,
                                    meta_data: None,
                                    market: None,
                                })
                            }
                        }
                        self.station.stations.sort_by_key(|a| a.name.clone());
                    }
                    EvmUpdate::StationMetaData(market_id, meta_data) => {
                        for station in &mut self.station.stations {
                            if station.market_id == market_id {
                                station.meta_data = Some(meta_data);
                                break;
                            }
                        }
                    }
                    EvmUpdate::StationCommodityListening(_market_id, _listenings) => {
                        todo!("Implement");
                    }
                    EvmUpdate::SystemMetaData(system_address, system_meta_data) => {
                        let length = self.explorer.systems.len();
                        for i in 0..length {
                            if self.explorer.systems[i].address == system_address {
                                self.explorer.systems[i]
                                    .name
                                    .clone_from(&system_meta_data.name);
                                self.explorer.systems[i]
                                    .allegiance
                                    .clone_from(&system_meta_data.allegiance);
                                self.explorer.systems[i]
                                    .government_localised
                                    .clone_from(&system_meta_data.government);
                                self.explorer.systems[i]
                                    .economy_localised
                                    .clone_from(&system_meta_data.economy);
                                self.explorer.systems[i]
                                    .second_economy_localised
                                    .clone_from(&system_meta_data.second_economy);
                                self.explorer.systems[i]
                                    .security_localised
                                    .clone_from(&system_meta_data.security);
                                self.explorer.systems[i]
                                    .population
                                    .clone_from(&system_meta_data.population);
                                self.explorer.systems[i]
                                    .body_count
                                    .clone_from(&system_meta_data.body_count.to_string());
                                self.explorer.systems[i]
                                    .x
                                    .clone_from(&system_meta_data.x);
                                self.explorer.systems[i]
                                    .y
                                    .clone_from(&system_meta_data.y);
                                self.explorer.systems[i]
                                    .z
                                    .clone_from(&system_meta_data.z);
                            }
                        }
                    }
                    EvmUpdate::PlanetList(system_address, planet_list) => {
                        let length = self.explorer.systems.len();
                        for i in 0..length {
                            if self.explorer.systems[i].address == system_address {
                                for body in &planet_list {
                                    if !body.get_signals().is_empty() {
                                        //Insert fresh body signals in system's planet signals
                                        let mut found = false;
                                        for signal in &self.explorer.systems[i].planet_signals {
                                            if signal.body_id == body.get_id() {
                                                found = true;
                                            }
                                        }
                                        if !found {
                                            self.explorer.systems[i].planet_signals.push(
                                                PlanetSignal {
                                                    body_name: body.get_name(),
                                                    body_id: body.get_id(),
                                                    signals: body.get_signals(),
                                                },
                                            );
                                        }
                                    }
                                }
                                self.explorer.systems[i].body_list.clone_from(&planet_list);
                            }
                        }
                    }
                }
            }
        }
    }

    fn is_station_registered(&self, station_identity: &StationIdentity) -> bool {
        for station in &self.station.stations {
            if station.market_id == station_identity.market_id {
                return true;
            }
            //List is sorted. If Ordering is greater, the station is not in the list
            if let Ordering::Less = station_identity.name.cmp(&station.name) {
                return false;
            }
        }
        false
    }
}
impl Default for EliteRustClient {
    fn default() -> Self {
        let log_path = initialize_logger();
        let settings = settings::Settings { log_path, ..Default::default() };
        let settings_pointer = Arc::new(settings.clone());

        info!("Starting...");
        info!("Current directory: {:?}", env::current_dir().unwrap());
        info!("Reading materials");

        let materials = MaterialState::default();

        info!("Starting threads");
        info!("Starting Evm Updater");
        let (evm_request_writer, evm_request_receiver) = mpsc::channel::<EvmRequest>();

        let mut evm_update_bus: Bus<EvmUpdate> = Bus::new(1);
        let evm_update_reader = evm_update_bus.add_rx();
        let settings_pointer_clone = settings_pointer.clone();
        thread::Builder::new()
            .name("edcas-evm-handler".into())
            .spawn(move || {
                let mut evm_updater = request_handler::initialize(
                    evm_update_bus,
                    evm_request_receiver,
                    settings_pointer_clone,
                );
                loop {
                    evm_updater.run_update();
                    sleep(Duration::from_secs(3));
                }
            })
            .expect("Failed to create thread jevm-handler");
        
        info!("Starting Journal reader");
        let mut journal_bus: Bus<JsonValue> = Bus::new(100);
        let journal_bus_reader = journal_bus.add_rx();
        let tangle_journal_bus_reader = journal_bus.add_rx();
        let settings_pointer_clone = settings_pointer.clone();
        thread::Builder::new()
            .name("edcas-journal-reader".into())
            .spawn(move || {
                let mut j_reader = journal_reader::initialize(settings_pointer_clone);
                loop {
                    //Sleep needed, because too frequent reads can lead to read the file while being written to it -> exception from json parser because json is not complete
                    sleep(Duration::from_millis(100));
                    j_reader.run(&mut journal_bus);
                }
            })
            .expect("Failed to create thread journal-reader");
        let settings_pointer_clone = settings_pointer.clone();
        
        info!(
            "Allow to share data over edcas: {}",
            settings_pointer.evm_settings.allow_share_data
        );
        if settings_pointer.evm_settings.allow_share_data {
            info!("Starting Evm Interpreter");
            //Buffer needs to be this large or in development, when the reader timeout is set to 0 the buffer can get full
            let settings_pointer = settings_pointer_clone;
            thread::Builder::new()
                .name("edcas-evm-interpreter".into())
                .spawn(move || {
                    let mut tangle_interpreter = backend::evm::journal_interpreter::initialize(
                        tangle_journal_bus_reader,
                        settings_pointer,
                    );
                    loop {
                        tangle_interpreter.run();
                    }
                })
                .expect("Failed to create thread evm-interpreter");
        }
        info!("Done starting threads");

        let cargo_reader = Arc::new(Mutex::new(cargo_reader::initialize(
            settings_pointer.clone(),
        )));
        let mining = mining::Mining {
            prospectors: Default::default(),
            cargo: cargo_reader.clone(),
        };

        Self {
            news: news::News::default(),
            about: about::About::default(),
            station: StationState {
                stations: vec![],
                search: "".to_string(),
                evm_request_writer: evm_request_writer.clone(),
                settings: settings_pointer.clone(),
            },
            carrier: CarrierState {
                carriers: vec![],
                search: "".to_string(),
                settings: settings_pointer.clone(),
            },
            explorer: explorer::Explorer {
                systems: vec![],
                index: 0,
                body_list_index: None,
                settings: settings_pointer.clone(),
            },
            state: News,
            cargo_reader,
            journal_log_bus_reader: journal_bus_reader,
            evm_request_writer,
            evm_update_reader,
            materials,
            settings,
            mining,
            timestamp: String::from(""),
        }
    }
}

pub enum State {
    News,
    About,
    StationPage,
    CarrierPage,
    Settings,
    Explorer,
    MaterialInventory,
    Mining,
}

fn initialize_logger() -> String {
    let mut log_directory = env::current_dir().unwrap().join("logs");
    if std::path::Path::new("/tmp/").exists() {
        log_directory = std::path::Path::new("/tmp/edcas-client/").to_path_buf();
    }

    if let Err(err) = fs::create_dir_all(&log_directory) {
        println!("Error while creating log directory: {:?}", err);
    }

    let log_filename = format!("{}.log", Local::now().format("%Y-%m-%d-%H-%M"));

    let log_path = log_directory.join(log_filename);
    //let log_path = log_file_path_buf.strip_prefix(&log_directory).unwrap_or(&log_file_path_buf);

    if let Ok(entries) = fs::read_dir(&log_directory) {
        let mut log_files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .collect();

        log_files.sort_by(|a, b| {
            b.metadata()
                .unwrap()
                .modified()
                .unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });

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

    let level = log::LevelFilter::Debug;

    let logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name(log_path.to_str().unwrap())
        .target_exclusions(&[
            "h2",
            "hyper",
            "rustls",
            "iota_wallet",
            "iota_client",
            "reqwest",
            "tree_builder",
            "html5ever",
            "ethers_providers",
        ])
        .level_filter(level);

    let _logger_output_config = fern_logger::LoggerOutputConfigBuilder::new()
        .name(log_path.to_str().unwrap())
        .target_exclusions(&["h2", "hyper", "rustls"])
        .level_filter(level);

    let config = fern_logger::LoggerConfig::build()
        .with_output(logger_output_config)
        .finish();
    fern_logger::logger_init(config).unwrap();
    log_path.to_str().unwrap().to_string()
}
