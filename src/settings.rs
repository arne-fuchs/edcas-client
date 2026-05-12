use tracing::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod appearance;
pub mod explorer;
pub mod grapic_editor;
pub mod icons;
pub mod journal_reader;
pub use icons::Icon;

use crate::settings::grapic_editor::GraphicEditorSettings;
use crate::settings::journal_reader::JournalReaderSettings;

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub appearance: appearance::AppearanceSettings,
    pub journal_reader: journal_reader::JournalReaderSettings,
    pub explorer: explorer::ExplorerSettings,
    pub graphics_editor: grapic_editor::GraphicEditorSettings,
    #[serde(default = "HashMap::default")]
    pub icons: HashMap<String, Icon>,
    #[serde(default = "HashMap::default")]
    pub stars: HashMap<String, Icon>,
    #[serde(default = "HashMap::default")]
    pub planets: HashMap<String, Icon>,
    /// Base URL of the edcas-eddn REST API, e.g. "http://localhost:3000"
    #[serde(default = "default_api_url")]
    pub api_url: String,
}

fn default_api_url() -> String {
    "https://edcas.de".into()
}

impl Default for Settings {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        return Settings::load_wasm();

        #[cfg(not(target_arch = "wasm32"))]
        Settings::load_native()
    }
}

// ─── WASM settings ────────────────────────────────────────────────────────────

#[cfg(target_arch = "wasm32")]
impl Settings {
    fn load_wasm() -> Self {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("edcas_settings").ok().flatten())
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_else(|| Settings {
                appearance: Default::default(),
                journal_reader: Default::default(),
                explorer: Default::default(),
                graphics_editor: Default::default(),
                icons: Default::default(),
                stars: Default::default(),
                planets: Default::default(),
                api_url: default_api_url(),
            })
    }

    pub fn save_wasm(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            if let Some(storage) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = storage.set_item("edcas_settings", &data);
            }
        }
    }
}

// ─── Native settings ──────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl Settings {
    fn load_native() -> Self {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;

        let mut settings_path = "settings.json".to_string();
        let mut settings_file = match std::env::var("HOME") {
            Ok(home) => {
                settings_path = format!("{}/.config/edcas-client/settings.json", home);
                if Path::new(&settings_path).exists() {
                    File::open(&settings_path).expect("Unable to open file")
                } else {
                    info!("Couldn't find config file in {} -> trying to create file structure and copy to desired config folder", settings_path);
                    create_setting_json(home)
                }
            }
            Err(err) => {
                warn!("{}", err);
                match std::fs::File::open("settings.json") {
                    Ok(file) => {
                        info!("Accessing settings file at settings.json");
                        file
                    }
                    Err(err) => {
                        warn!("{}", err);
                        if Path::new("/etc/edcas-client/settings-example.json").exists() {
                            info!("Copying from /etc/edcas-client/settings-example.json to settings.json");
                            std::fs::copy("/etc/edcas-client/settings-example.json", "settings.json")
                                .expect("Couldn't copy settings file from /etc/edcas-client/settings-example.json to local settings.json");
                        } else {
                            std::fs::copy("settings-example.json", "settings.json")
                                .expect("Couldn't copy settings file to $HOME/.config/edcas-client/");
                        }
                        info!("Accessing settings file at settings.json");
                        std::fs::File::open("settings.json").unwrap()
                    }
                }
            }
        };

        let mut json_string = String::new();
        settings_file.read_to_string(&mut json_string).unwrap();
        let mut settings: Settings =
            serde_json::from_str(&json_string).expect("Invalid json settings file");

        if !Path::new(&settings.journal_reader.journal_directory).exists() {
            warn!("journal path {} does not exists", settings.journal_reader.journal_directory);
            settings.journal_reader = JournalReaderSettings::default();
        }
        debug!("Journal logs: {}", &settings.journal_reader.journal_directory);

        if !Path::new(&settings.graphics_editor.graphics_directory).exists() {
            warn!("graphics path {} does not exists", &settings.graphics_editor.graphics_directory);
            settings.graphics_editor = GraphicEditorSettings::default();
        } else if cfg!(target_os = "windows") {
            settings.graphics_editor.graphic_override_content = format!(
                "{}\\GraphicsConfigurationOverride.xml",
                &settings.graphics_editor.graphic_override_content
            );
        } else if cfg!(target_os = "linux") {
            settings.graphics_editor.graphic_override_content = format!(
                "{}/GraphicsConfigurationOverride.xml",
                &settings.graphics_editor.graphic_override_content
            );
        }
        debug!("Graphics path: {}", &settings.graphics_editor.graphics_directory);

        settings
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn create_setting_json(home: String) -> std::fs::File {
    use std::fs::File;

    match std::fs::create_dir_all(format!("{}/.config/edcas-client", home)) {
        Ok(_) => {
            info!("Created $HOME/.config/edcas-client/");
            info!("Copying from /etc/edcas-client/settings-example.json to $HOME/.config/edcas-client/settings.json");
            match std::fs::copy(
                "/etc/edcas-client/settings-example.json",
                format!("{}/.config/edcas-client/settings.json", home),
            ) {
                Ok(_) => {
                    info!("Copied /etc/edcas-client/settings-example.json to {}",
                        format!("{}/.config/edcas-client/settings.json", home));
                }
                Err(err) => {
                    info!("Failed copying from /etc/edcas-client/settings-example.json\n Trying to copy from settings-example.json: {}", err);
                    std::fs::copy(
                        "settings-example.json",
                        format!("{}/.config/edcas-client/settings.json", home),
                    ).expect("Couldn't copy settings file to $HOME/.config/edcas-client/");
                }
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                info!("Setting permissions: {:?}",
                    std::fs::set_permissions(
                        format!("{}/.config/edcas-client/settings.json", home),
                        std::fs::Permissions::from_mode(0o644)
                    )
                );
            }
            info!("Accessing settings file at $HOME/.config/edcas-client/settings.json");
            std::fs::File::open(format!("{}/.config/edcas-client/settings.json", home))
                .expect("Couldn't open settings file in $HOME/.config/edcas-client/")
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
                    match std::fs::copy("settings-example.json", "settings.json") {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error copying settings file: {}", err);
                            panic!("Error copying settings file: {}", err);
                        }
                    }
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        info!("Setting permissions: {:?}",
                            std::fs::set_permissions(
                                "settings.json",
                                std::fs::Permissions::from_mode(0o644)
                            )
                        );
                    }
                    info!("Accessing settings file at settings.json");
                    File::open("settings.json").unwrap()
                }
            }
        }
    }
}
