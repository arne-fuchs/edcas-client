use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Persistent state for fleet carriers the player has marked as "mine".
/// Stored in `my_carriers.json` alongside `pins.json`.
/// A carrier present as a key (even with empty cargo) is considered "mine".
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MyCarriersData {
    /// market_id → commodity (normalised lower-case) → count
    #[serde(default)]
    pub carriers: HashMap<i64, HashMap<String, i32>>,
}

impl MyCarriersData {
    pub fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = Self::native_path();
            let loaded: Self = std::fs::read_to_string(&path)
                .ok()
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default();
            // One-time migration: if this file is brand-new, pull IDs from pins.json
            if loaded.carriers.is_empty() {
                let pins = crate::pins::Pins::load();
                if !pins.my_carriers.is_empty() {
                    let migrated = Self {
                        carriers: pins
                            .my_carriers
                            .into_iter()
                            .map(|id| (id, HashMap::new()))
                            .collect(),
                    };
                    migrated.save();
                    return migrated;
                }
            }
            loaded
        }
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
                .and_then(|s| s.get_item("edcas_my_carriers").ok().flatten())
                .and_then(|data| serde_json::from_str(&data).ok())
                .unwrap_or_default()
        }
    }

    pub fn save(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
                    let _ = storage.set_item("edcas_my_carriers", &data);
                }
            }
        }
    }

    pub fn market_ids(&self) -> HashSet<i64> {
        self.carriers.keys().copied().collect()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_path() -> std::path::PathBuf {
        crate::settings::config_dir().join("my_carriers.json")
    }
}
