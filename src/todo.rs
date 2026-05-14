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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoList {
    pub items: Vec<TodoItem>,
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

    #[cfg(not(target_arch = "wasm32"))]
    fn native_path() -> std::path::PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            std::path::PathBuf::from(home).join(".config/edcas-client/todo.json")
        } else {
            std::path::PathBuf::from("todo.json")
        }
    }
}
