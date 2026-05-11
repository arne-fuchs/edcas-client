use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppearanceSettings {
    #[serde(default = "default_color")]
    pub color: String,
}

fn default_color() -> String {
    "orange".to_string()
}
