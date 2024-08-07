use bus::BusReader;
use std::default::Default;
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};

use crate::edcas::backend::evm::edcas_contract;
use crate::edcas::backend::evm::journal_interpreter::Edcas;
use eframe::egui;
use eframe::egui::{Color32, RichText};
use eframe::epaint::ahash::HashMap;
use ethers::addressbook::Address;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::*;
use ethers::utils::hex;
use log::{error, info, warn};
use serde_json::json;

use crate::edcas::settings::ActionAtShutdownSignal::{Continue, Exit, Nothing};

pub(crate) mod presets;

pub struct Icon {
    pub name: String,
    pub char: String,
    pub color: Color32,
    pub enabled: bool,
}

impl Icon {
    pub fn get_richtext(&self) -> RichText {
        RichText::new(&self.char).color(self.color)
    }
}

pub struct AppearanceSettings {
    pub font_size: f32,
    pub font_style: String,
    pub font_id: egui::FontId,
    pub applied: bool,
}

pub struct JournalReaderSettings {
    pub journal_directory: String,
    pub action_at_shutdown_signal: ActionAtShutdownSignal,
}

#[derive(Clone)]
pub enum ActionAtShutdownSignal {
    Exit,
    Nothing,
    Continue,
}

impl FromStr for ActionAtShutdownSignal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Exit" => Ok(Exit),
            "nothing" => Ok(Nothing),
            "continue" => Ok(Continue),
            _ => Err("Failed to parse ActionShutdownSignal".to_string()),
        }
    }
}

impl Display for ActionAtShutdownSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Exit => "Exit".to_string(),
            Nothing => "nothing".to_string(),
            Continue => "continue".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl PartialEq for ActionAtShutdownSignal {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

pub struct ExplorerSettings {
    pub include_system_name: bool,
}

pub struct EvmSettings {
    pub url: String,
    pub n_timeout: u64,
    pub n_attempts: u64,
    pub allow_share_data: bool,
    pub private_key: String,
    pub smart_contract_address: String,
    pub contract: Option<Edcas>,
    pub show_upload_data_window: bool,
    pub journal_read_status: Option<JournalReadStatus>,
}

pub struct JournalReadStatus {
    pub current_log: u32,
    pub total_logs: u32,
    pub log_index_updates: BusReader<i64>,
}

pub struct GraphicEditorSettings {
    pub graphics_directory: String,
    pub graphic_override_content: String,
    pub show_editor: bool,
}

pub struct Settings {
    pub appearance_settings: AppearanceSettings,
    pub journal_reader_settings: JournalReaderSettings,
    pub explorer_settings: ExplorerSettings,
    pub evm_settings: EvmSettings,
    pub graphic_editor_settings: GraphicEditorSettings,
    pub icons: HashMap<String, Icon>,
    pub stars: HashMap<String, Icon>,
    pub planets: HashMap<String, Icon>,
    pub settings_path: String,
    pub log_path: String,
}

impl Default for Settings {
    fn default() -> Self {
        let mut settings_path = "settings.json".to_string();
        let mut settings_file = match env::var("HOME") {
            Ok(home) => match File::open(format!("{}/.config/edcas-client/settings.json", home)) {
                Ok(file) => {
                    settings_path = format!("{}/.config/edcas-client/settings.json", home);
                    file
                }
                Err(err) => {
                    warn!("{}", err);
                    info!("Couldn't find config file in {} -> trying to create file structure and copy to desired config folder",format!("{}/.config/edcas-client/settings.json",home));
                    match fs::create_dir_all(format!("{}/.config/edcas-client", home)) {
                        Ok(_) => {
                            info!("Created $HOME/.config/edcas-client/");
                            info!("Copying from /etc/edcas-client/settings-example.json to $HOME/.config/edcas-client/settings.json");
                            match fs::copy(
                                "/etc/edcas-client/settings-example.json",
                                format!("{}/.config/edcas-client/settings.json", home),
                            ) {
                                Ok(_) => {
                                    info!(
                                        "Copied /etc/edcas-client/settings-example.json to {}",
                                        format!("{}/.config/edcas-client/settings.json", home)
                                    );
                                }
                                Err(err) => {
                                    info!("Failed copying from /etc/edcas-client/settings-example.json\n Trying to copy from settings-example.json to $HOME/.config/edcas-client/settings.json: {}",err);
                                    fs::copy("settings-example.json", format!("{}/.config/edcas-client/settings.json", home)).expect("Couldn't copy settings file to $HOME/.config/edcas-client/");
                                }
                            }
                            #[cfg(unix)]
                            {
                                info!(
                                    "Setting permissions: {:?}",
                                    fs::set_permissions(
                                        format!("{}/.config/edcas-client/settings.json", home),
                                        fs::Permissions::from_mode(0o644)
                                    )
                                );
                            }

                            info!("Accessing settings file at $HOME/.config/edcas-client/settings.json");
                            settings_path = format!("{}/.config/edcas-client/settings.json", home);
                            File::open(format!("{}/.config/edcas-client/settings.json", home))
                                .expect(
                                    "Couldn't open settings file in $HOME/.config/edcas-client/",
                                )
                        }
                        Err(err) => {
                            warn!("{}", err);
                            info!("Couldn't create directories in $HOME/.config/edcas-client/");
                            match File::open("settings.json") {
                                Ok(file) => {
                                    info!("Accessing settings file at settings.json");
                                    file
                                }
                                Err(err) => {
                                    warn!("{}", err);
                                    info!("Copying from settings-example.json to settings.json");
                                    match fs::copy("settings-example.json", "settings.json") {
                                        Ok(_) => {}
                                        Err(err) => {
                                            error!("Error copying settings file: {}", err);
                                            panic!("Error copying settings file: {}", err);
                                        }
                                    }
                                    #[cfg(unix)]
                                    {
                                        info!(
                                            "Setting permissions: {:?}",
                                            fs::set_permissions(
                                                "settings.json",
                                                fs::Permissions::from_mode(0o644)
                                            )
                                        );
                                    }
                                    #[cfg(windows)]
                                    {
                                        fs::metadata("settings.json")
                                            .unwrap()
                                            .permissions()
                                            .set_readonly(false);
                                    }
                                    info!("Accessing settings file at settings.json");
                                    File::open("settings.json").unwrap()
                                }
                            }
                        }
                    }
                }
            },
            Err(err) => {
                warn!("{}", err);
                match File::open("settings.json") {
                    Ok(file) => {
                        info!("Accessing settings file at settings.json");
                        file
                    }
                    Err(err) => {
                        warn!("{}", err);
                        info!(
                            "Copying from /etc/edcas-client/settings-example.json to settings.json"
                        );
                        fs::copy("/etc/edcas-client/settings-example.json", "settings.json")
                            .unwrap_or(fs::copy("settings-example.json", "settings.json").expect(
                                "Couldn't copy settings file to $HOME/.config/edcas-client/",
                            ));
                        info!("Accessing settings file at settings.json");
                        File::open("settings.json").unwrap()
                    }
                }
            }
        };

        let mut json_string: String = String::from("");
        settings_file.read_to_string(&mut json_string).unwrap();
        let json = json::parse(&json_string).unwrap();

        //---------------------------
        // Journal reader
        //---------------------------

        let mut journal_directory = json["journal-reader"]["directory"].to_string();
        let journal_path = Path::new(&journal_directory);
        if !journal_path.exists() {
            if cfg!(target_os = "windows") {
                let mut userprofile = env::var("USERPROFILE").unwrap_or("".to_string());
                userprofile.push_str("\\Saved Games\\Frontier Developments\\Elite Dangerous");
                journal_directory = userprofile;
            } else if cfg!(target_os = "linux") {
                let mut home = env::var("HOME").unwrap_or("~".to_string());
                home.push_str("/.steam/steam/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/Saved Games/Frontier Developments/Elite Dangerous");
                journal_directory = home;
            }
            if !Path::new(&journal_directory).exists() {
                journal_directory = String::from("");
            }
            println!("Journal logs: {}", &journal_directory);
        }

        //---------------------------
        // Graphics directory
        //---------------------------
        let mut graphics_directory = json["graphics-editor"]["graphics-directory"].to_string();
        let graphics_path = Path::new(&graphics_directory);
        let mut graphics_override_file = format!("{}/GraphicsConfigurationOverride.xml", graphics_directory);
        if !graphics_path.exists() {
            if cfg!(target_os = "windows") {
                let mut userprofile = env::var("USERPROFILE").unwrap_or("".to_string());
                userprofile.push_str(
                    "\\AppData\\Local\\Frontier Developments\\Elite Dangerous\\Options\\Graphics",
                );
                graphics_directory = userprofile;
                graphics_override_file =
                    format!("{}\\GraphicsConfigurationOverride.xml", graphics_directory);
            } else if cfg!(target_os = "linux") {
                let mut home = env::var("HOME").unwrap_or("~".to_string());
                home.push_str("/.steam/root/steamapps/compatdata/359320/pfx/drive_c/users/steamuser/AppData/Local/Frontier Developments/Elite Dangerous/Options/Graphics");
                graphics_directory = home;
                graphics_override_file =
                    format!("{}/GraphicsConfigurationOverride.xml", graphics_directory);
            }
            if !Path::new(&graphics_directory).exists() {
                graphics_directory = String::from("");
            }
            info!("Graphics path: {}", &graphics_directory);
        }

        //---------------------------
        // EVM
        //---------------------------

        let mut private_key = json["evm"]["private-key"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if private_key.is_empty() {
            let mut buffer = [0u8; 32];

            // Open a file to read random bytes from the OS
            #[cfg(target_os = "linux")]
            let mut file = File::open("/dev/urandom").unwrap();

            #[cfg(target_os = "windows")]
            let mut file = File::open("C:\\Windows\\System32\\advapi32.dll").unwrap();

            // Read random bytes into the buffer
            file.read_exact(&mut buffer).unwrap();

            private_key = hex::encode(buffer);
        }

        let evm_url = json["evm"]["base-url"]
            .as_str()
            .unwrap_or("https://api.testnet.undertheocean.net/wasp/api/v1/chains/rms1prflcwju7wyzks0wzyyvejz6sqzf8gl7qwyen339e7zeaue9k99yv2exr04/evm")
            .to_string();

        let smart_contract_address = json["evm"]["smart-contract-address"]
            .as_str()
            .unwrap_or("0xe346ac7a39d1b7a62b318d901ffac36dfcfb277f")
            .to_string();

        let middleware_result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let provider = Provider::connect(evm_url.as_str()).await;

                let wallet: LocalWallet = private_key.parse::<LocalWallet>().unwrap();

                let mut result =
                    SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
                        .await;
                let mut retries = 0;
                while result.is_err() && retries < 20 {
                    retries += 1;
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    result =
                        SignerMiddleware::new_with_provider_chain(provider.clone(), wallet.clone())
                            .await;
                }
                result
            });

        let contract = match middleware_result {
            Ok(middleware) => {
                let edcas_address = smart_contract_address.parse::<Address>().unwrap();
                Some(edcas_contract::EDCAS::new(
                    edcas_address,
                    Arc::new(middleware),
                ))
            }
            Err(_) => {
                error!("Couldn't get contract by settings initialization");
                None
            }
        };

        //---------------------------
        // Appearance
        //---------------------------

        let font_size = json["appearance"]["font-size"].as_f32().unwrap_or(24.0);
        let mut _font_id = egui::FontId::default();

        match json["appearance"]["font-family"]
            .as_str()
            .unwrap_or("Proportional")
        {
            "Monospace" => {
                _font_id = egui::FontId::monospace(font_size);
            }
            "Proportional" | _ => {
                _font_id = egui::FontId::proportional(font_size);
            }
        }

        //---------------------------
        // Icons
        //---------------------------
        let mut icons: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["icons"].len() {
            let icon_json = &json["icons"][i];
            icons.insert(
                icon_json["name"].to_string(),
                Icon {
                    name: icon_json["name"].to_string(),
                    char: icon_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(
                        icon_json["r"].as_u8().unwrap_or(255),
                        icon_json["g"].as_u8().unwrap_or(255),
                        icon_json["b"].as_u8().unwrap_or(255),
                    ),
                    enabled: icon_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }

        //---------------------------
        // Stars
        //---------------------------
        let mut stars: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["stars"].len() {
            let stars_json = &json["stars"][i];
            stars.insert(
                stars_json["class"].to_string(),
                Icon {
                    name: stars_json["class"].to_string(),
                    char: stars_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(
                        stars_json["r"].as_u8().unwrap_or(255),
                        stars_json["g"].as_u8().unwrap_or(255),
                        stars_json["b"].as_u8().unwrap_or(255),
                    ),
                    enabled: stars_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }

        //---------------------------
        // Planets
        //---------------------------
        let mut planets: HashMap<String, Icon> = HashMap::default();
        for i in 0..json["planets"].len() {
            let stars_json = &json["planets"][i];
            planets.insert(
                stars_json["class"].to_string(),
                Icon {
                    name: stars_json["class"].to_string(),
                    char: stars_json["char"].as_str().unwrap_or("⁉").to_string(),
                    color: Color32::from_rgb(
                        stars_json["r"].as_u8().unwrap_or(255),
                        stars_json["g"].as_u8().unwrap_or(255),
                        stars_json["b"].as_u8().unwrap_or(255),
                    ),
                    enabled: stars_json["enabled"].as_bool().unwrap_or(true),
                },
            );
        }

        Self {
            appearance_settings: AppearanceSettings {
                font_size: json["appearance"]["font-size"].as_f32().unwrap_or(24.0),
                font_style: json["appearance"]["font_style"]
                    .as_str()
                    .unwrap_or("Proportional")
                    .to_string(),
                font_id: _font_id,
                applied: false,
            },
            journal_reader_settings: JournalReaderSettings {
                journal_directory,
                action_at_shutdown_signal: ActionAtShutdownSignal::from_str(
                    json["journal-reader"]["action-at-shutdown-signal"]
                        .as_str()
                        .unwrap_or("Nothing"),
                )
                .unwrap_or(Nothing),
            },
            explorer_settings: ExplorerSettings {
                include_system_name: json["explorer"]["include_system_name"]
                    .as_bool()
                    .unwrap_or(true),
            },
            evm_settings: EvmSettings {
                url: evm_url,
                n_timeout: json["evm"]["timeout"].as_u64().unwrap_or(5),
                n_attempts: json["evm"]["attempts"].as_u64().unwrap_or(4),
                allow_share_data: json["evm"]["allow-share-data"].as_bool().unwrap_or(false),
                private_key,
                smart_contract_address,
                contract,
                show_upload_data_window: false,
                journal_read_status: None,
            },
            graphic_editor_settings: GraphicEditorSettings {
                graphics_directory: graphics_directory.clone(),
                graphic_override_content: fs::read_to_string(graphics_override_file)
                    .unwrap_or_default(),
                show_editor: false,
            },
            icons,
            stars,
            planets,
            settings_path,
            log_path: "".to_string(),
        }
    }
}

impl Settings {
    pub fn save_settings_to_file(&mut self) {
        let icon_array: serde_json::Value = self
            .icons
            .values()
            .map(|icon| {
                json!(
                    {
                        "name": icon.name,
                        "char": icon.char,
                        "r": icon.color.r(),
                        "g": icon.color.g(),
                        "b": icon.color.b(),
                        "enabled": icon.enabled
                    }
                )
            })
            .collect();
        let star_array: serde_json::Value = self
            .stars
            .values()
            .map(|star| {
                json!(
                    {
                        "class": star.name,
                        "char": star.char,
                        "r": star.color.r(),
                        "g": star.color.g(),
                        "b": star.color.b(),
                        "enabled": star.enabled
                    }
                )
            })
            .collect();
        let planet_array: serde_json::Value = self
            .planets
            .values()
            .map(|planet| {
                json!(
                    {
                        "class": planet.name,
                        "char": planet.char,
                        "r": planet.color.r(),
                        "g": planet.color.g(),
                        "b": planet.color.b(),
                        "enabled": planet.enabled
                    }
                )
            })
            .collect();

        let json = json!(
            {
                "appearance": {
                    "font-size": self.appearance_settings.font_size,
                    "font-family": self.appearance_settings.font_id.family.to_string(),
                },
                "journal-reader": {
                    "directory": self.journal_reader_settings.journal_directory,
                    "action-at-shutdown-signal": self.journal_reader_settings.action_at_shutdown_signal.to_string()
                },
                "explorer": {
                    "include_system_name": self.explorer_settings.include_system_name
                },
                "evm": {
                    "base-url": self.evm_settings.url,
                    "timeout": self.evm_settings.n_timeout,
                    "attempts": self.evm_settings.n_attempts,
                    "private-key": self.evm_settings.private_key,
                    "smart-contract-address": self.evm_settings.smart_contract_address,
                    "allow-share-data": self.evm_settings.allow_share_data
                },
                "icons": icon_array,
                "stars": star_array,
                "planets": planet_array,
                "graphics-editor": {
                    "graphics-directory": self.graphic_editor_settings.graphics_directory
                }
            }
        );
        info!("Trying to write settings file to {}", &self.settings_path);
        match File::create(&self.settings_path) {
            Ok(mut settings_file) => {
                match settings_file
                    .write_all(serde_json::to_string_pretty(&json).unwrap().as_bytes())
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Failed to save settings: {}", err);
                        panic!("Failed to save settings: {}", err);
                    }
                };
            }
            Err(err) => {
                error!("Failed to create settings: {}", err);
                panic!("Failed to create settings: {}", err);
            }
        };
        info!("Done writing to settings file");
    }
}
