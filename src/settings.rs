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
    /// Whether to upload journal data to the edcas API. Default: true.
    #[serde(default = "default_true")]
    pub edcas_api_enabled: bool,
    /// Whether to upload journal data to the EDDN network. Default: true.
    #[serde(default = "default_true")]
    pub eddn_enabled: bool,
    /// EDDN upload gateway URL.
    #[serde(default = "default_eddn_url")]
    pub eddn_url: String,
    /// When true, EDDN messages are sent to the test pipeline (`/test` schemaRef suffix).
    #[serde(default = "default_true")]
    pub eddn_test_mode: bool,
}

fn default_api_url() -> String {
    "https://edcas.de".into()
}

fn default_true() -> bool {
    true
}

fn default_eddn_url() -> String {
    "https://eddn.edcd.io:4430/upload/".into()
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
                edcas_api_enabled: true,
                eddn_enabled: true,
                eddn_url: default_eddn_url(),
                eddn_test_mode: true,
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

// ─── Config directory ─────────────────────────────────────────────────────────

/// Returns the OS-appropriate config directory for edcas-client.
///
/// - Windows: `%APPDATA%\edcas-client`
/// - Linux/macOS: `$XDG_CONFIG_HOME/edcas-client` or `~/.config/edcas-client`
#[cfg(not(target_arch = "wasm32"))]
pub fn config_dir() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    if let Ok(appdata) = std::env::var("APPDATA") {
        return std::path::PathBuf::from(appdata).join("edcas-client");
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return std::path::PathBuf::from(xdg).join("edcas-client");
    }
    if let Ok(home) = std::env::var("HOME") {
        return std::path::PathBuf::from(home).join(".config").join("edcas-client");
    }
    std::path::PathBuf::from("edcas-client-config")
}

// ─── Native settings ──────────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
impl Settings {
    /// The edcas-API base URL to upload to, or `None` when uploading is disabled or unset.
    pub fn edcas_api_url(&self) -> Option<String> {
        let url = self.api_url.trim();
        if self.edcas_api_enabled && !url.is_empty() {
            Some(url.to_string())
        } else {
            None
        }
    }

    /// The EDDN uploader configuration, or `None` when EDDN uploads are disabled.
    pub fn eddn_config(&self) -> Option<crate::eddn::EddnConfig> {
        if self.eddn_enabled && !self.eddn_url.trim().is_empty() {
            Some(crate::eddn::EddnConfig {
                url: self.eddn_url.trim().to_string(),
                test_mode: self.eddn_test_mode,
            })
        } else {
            None
        }
    }

    fn load_native() -> Self {
        use std::fs::File;
        use std::io::Read;
        use std::path::Path;

        let cfg_dir = config_dir();
        let settings_path = cfg_dir.join("settings.json");

        let mut settings_file = if settings_path.exists() {
            info!("Accessing settings file at {}", settings_path.display());
            File::open(&settings_path).expect("Unable to open settings file")
        } else {
            info!("Settings file not found, creating at {}", settings_path.display());
            create_settings_in(&cfg_dir)
        };

        let mut json_string = String::new();
        settings_file.read_to_string(&mut json_string).unwrap();
        let mut settings: Settings =
            serde_json::from_str(&json_string).expect("Invalid json settings file");

        if !Path::new(&settings.journal_reader.journal_directory).exists() {
            warn!("journal path {} does not exist", settings.journal_reader.journal_directory);
            settings.journal_reader = JournalReaderSettings::default();
        }
        debug!("Journal logs: {}", &settings.journal_reader.journal_directory);

        if !Path::new(&settings.graphics_editor.graphics_directory).exists() {
            warn!("graphics path {} does not exist", &settings.graphics_editor.graphics_directory);
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

/// Creates the config directory and copies a settings template into it.
/// Falls back to `settings.json` in the current directory if the dir can't be created.
#[cfg(not(target_arch = "wasm32"))]
fn create_settings_in(cfg_dir: &std::path::Path) -> std::fs::File {
    use std::fs::File;

    let dest = cfg_dir.join("settings.json");

    if std::fs::create_dir_all(cfg_dir).is_ok() {
        // Prefer system-installed template, fall back to bundled one.
        let src = if std::path::Path::new("/etc/edcas-client/settings-example.json").exists() {
            "/etc/edcas-client/settings-example.json"
        } else {
            "settings-example.json"
        };
        if let Err(e) = std::fs::copy(src, &dest) {
            warn!("Could not copy settings template: {}", e);
        } else {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o644));
            }
            info!("Created settings file at {}", dest.display());
        }
        if dest.exists() {
            return File::open(&dest).expect("Could not open new settings file");
        }
    } else {
        warn!("Could not create config dir {}", cfg_dir.display());
    }

    // Last resort: use settings.json in the current directory.
    if !std::path::Path::new("settings.json").exists() {
        let src = if std::path::Path::new("/etc/edcas-client/settings-example.json").exists() {
            "/etc/edcas-client/settings-example.json"
        } else {
            "settings-example.json"
        };
        std::fs::copy(src, "settings.json").expect("Could not create settings.json");
    }
    info!("Falling back to settings.json in current directory");
    File::open("settings.json").expect("Could not open settings.json")
}
