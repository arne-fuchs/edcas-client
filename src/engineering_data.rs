use serde::Deserialize;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct EngineerInfo {
    pub id: String,
    pub name: String,
    pub system: String,
    pub station: String,
    pub specialties: Vec<String>,
    pub unlock: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MaterialCost {
    pub name: String,
    pub count: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Modification {
    pub id: String,
    pub name: String,
    pub module_type: String,
    pub engineer_ids: Vec<String>,
    pub effect: String,
    pub max_grade: u8,
    pub grades: std::collections::HashMap<String, Vec<MaterialCost>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineersFile {
    pub ship: Vec<EngineerInfo>,
    pub onfoot: Vec<EngineerInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModificationsFile {
    pub ship: Vec<Modification>,
    pub onfoot: Vec<Modification>,
}

static ENGINEERS: OnceLock<EngineersFile> = OnceLock::new();
static MODIFICATIONS: OnceLock<ModificationsFile> = OnceLock::new();

const ENGINEERS_JSON: &str = include_str!("../engineers.json");
const MODIFICATIONS_JSON: &str = include_str!("../modifications.json");

pub fn engineers() -> &'static EngineersFile {
    ENGINEERS.get_or_init(|| {
        serde_json::from_str(ENGINEERS_JSON).expect("engineers.json is invalid")
    })
}

pub fn modifications() -> &'static ModificationsFile {
    MODIFICATIONS.get_or_init(|| {
        serde_json::from_str(MODIFICATIONS_JSON).expect("modifications.json is invalid")
    })
}
