use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModKind {
    Ship,
    OnFoot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub mod_id: String,
    pub grade: u8,
    pub mod_name: String,
    pub module_type: String,
    pub kind: ModKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionTodoResource {
    pub commodity_name: String,
    pub display_name: String,
    pub required_amount: i32,
    pub provided_amount: i32,
    pub payment: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstructionTodoItem {
    pub market_id: i64,
    pub station_name: String,
    pub system_name: String,
    pub resources: Vec<ConstructionTodoResource>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
    #[serde(default)]
    pub construction_items: Vec<ConstructionTodoItem>,
}

impl TodoList {
    pub fn load() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        {
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
                .and_then(|s| s.get_item("edcas_todo").ok().flatten())
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
                    let _ = storage.set_item("edcas_todo", &data);
                }
            }
        }
    }

    pub fn add(&mut self, item: TodoItem) {
        let already = self.items.iter().any(|i| i.mod_id == item.mod_id && i.grade == item.grade);
        if !already {
            self.items.push(item);
            self.save();
        }
    }

    pub fn remove(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.items.remove(idx);
            self.save();
        }
    }

    pub fn add_construction_item(&mut self, item: ConstructionTodoItem) {
        if !self.construction_items.iter().any(|i| i.market_id == item.market_id) {
            self.construction_items.push(item);
            self.save();
        }
    }

    pub fn remove_construction_item(&mut self, market_id: i64) {
        let before = self.construction_items.len();
        self.construction_items.retain(|i| i.market_id != market_id);
        if self.construction_items.len() != before {
            self.save();
        }
    }

    /// Update a pinned construction item's resource snapshot (e.g. after re-docking).
    pub fn update_construction_item(&mut self, item: ConstructionTodoItem) {
        if let Some(existing) = self.construction_items.iter_mut().find(|i| i.market_id == item.market_id) {
            *existing = item;
            self.save();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn native_path() -> std::path::PathBuf {
        crate::settings::config_dir().join("todo.json")
    }
}
