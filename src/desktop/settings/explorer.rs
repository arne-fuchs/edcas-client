use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ExplorerSettings {
    pub include_system_name: bool,
}
