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
    /// ISO 8601 timestamp of the last journal event processed when this snapshot was saved.
    /// Used to skip already-counted CargoTransfer events when replaying the same journal file
    /// after an in-session restart.
    #[serde(default)]
    pub snapshot_timestamp: String,
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
                        snapshot_timestamp: String::new(),
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
                // Strip zero-count entries before serializing — they're equivalent to absent.
                let clean = Self {
                    carriers: self.carriers.iter()
                        .map(|(id, cargo)| {
                            let trimmed: std::collections::HashMap<String, i32> =
                                cargo.iter().filter(|(_, &v)| v != 0).map(|(k, &v)| (k.clone(), v)).collect();
                            (*id, trimmed)
                        })
                        .collect(),
                    snapshot_timestamp: self.snapshot_timestamp.clone(),
                };
                if let Ok(data) = serde_json::to_string_pretty(&clean) {
                    let tmp = parent.join("my_carriers.json.tmp");
                    if std::fs::write(&tmp, &data).is_ok() {
                        let _ = std::fs::rename(&tmp, &path);
                    }
                }
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
