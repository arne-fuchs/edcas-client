use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ExplorerSettings {
    pub include_system_name: bool,
}
