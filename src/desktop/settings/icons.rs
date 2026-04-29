use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Icon {
    pub char: String,
    pub color: String,
    pub enabled: bool,
}
