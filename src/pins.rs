use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pins {
    #[serde(default)]
    pub stations: HashSet<i64>,
    #[serde(default)]
    pub carriers: HashSet<i64>,
    #[serde(default)]
    pub factions: HashSet<String>,
    #[serde(default)]
    pub constructions: HashSet<i64>,
}

impl Pins {
    pub fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::path::PathBuf;
            let path = Self::native_path();
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        }
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("edcas_pins").ok().flatten())
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        }
    }

    pub fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::path::PathBuf;
            let path = Self::native_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            if let Ok(data) = serde_json::to_string_pretty(self) {
                let _ = std::fs::write(path, data);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(data) = serde_json::to_string(self) {
                if let Some(storage) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = storage.set_item("edcas_pins", &data);
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_path() -> std::path::PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config/edcas-client/pins.json")
        } else {
            std::path::PathBuf::from("pins.json")
        }
    }
}
