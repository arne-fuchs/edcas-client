use std::fs;

use eframe::epaint::ahash::HashMap;
use eframe::Frame;
use json::JsonValue;
use log::info;

pub struct MaterialState {
    pub raw: HashMap<String, Material>,
    pub manufactured: HashMap<String, Material>,
    pub encoded: HashMap<String, Material>,
    pub showing: Option<Material>,
    pub search: String,
}

impl Default for MaterialState {
    fn default() -> Self {
        let mut materials = MaterialState {
            raw: HashMap::default(),
            manufactured: HashMap::default(),
            encoded: HashMap::default(),
            showing: None,
            search: "".to_string(),
        };
        info!("Looking for material file in /usr/share/edcas-client/materials.json");
        let materials_content = match fs::read_to_string("/usr/share/edcas-client/materials.json") {
            Ok(content) => {
                info!("Material file found");
                content
            }
            Err(_) => {
                info!("Material file not found -> looking in the local folder");
                fs::read_to_string("materials.json").unwrap()
            }
        };
        let materials_json = json::parse(materials_content.as_str()).unwrap();

        let encoded_array = &materials_json["encoded"];
        for i in 0..encoded_array.len() {
            let encoded = &encoded_array[i];

            let locations: Vec<String> = get_array_values(&encoded, "locations");

            let sources: Vec<String> = get_array_values(&encoded, "sources");

            let engineering: Vec<String> = get_array_values(&encoded, "engineering");

            let synthesis: Vec<String> = get_array_values(&encoded, "synthesis");

            materials.encoded.insert(
                encoded["name"].to_string(),
                Material {
                    name: encoded["name"].to_string(),
                    name_localised: encoded["name_localised"].to_string(),
                    grade: encoded["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: encoded["maximum"].as_u64().unwrap(),
                    category: encoded["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: encoded["description"].to_string(),
                },
            );
        }

        let manufactured_array = &materials_json["manufactured"];
        for i in 0..manufactured_array.len() {
            let manufactured = &manufactured_array[i];

            let locations: Vec<String> = get_array_values(&manufactured, "locations");

            let sources: Vec<String> = get_array_values(&manufactured, "sources");

            let engineering: Vec<String> = get_array_values(&manufactured, "engineering");

            let synthesis: Vec<String> = get_array_values(&manufactured, "synthesis");

            materials.manufactured.insert(
                manufactured["name"].to_string(),
                Material {
                    name: manufactured["name"].to_string(),
                    name_localised: manufactured["name_localised"].to_string(),
                    grade: manufactured["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: manufactured["maximum"].as_u64().unwrap(),
                    category: manufactured["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: manufactured["description"].to_string(),
                },
            );
        }

        let raw_array = &materials_json["raw"];
        for i in 0..raw_array.len() {
            let raw = &raw_array[i];

            let locations: Vec<String> = get_array_values(&raw, "locations");

            let sources: Vec<String> = get_array_values(&raw, "sources");

            let engineering: Vec<String> = get_array_values(&raw, "engineering");

            let synthesis: Vec<String> = get_array_values(&raw, "synthesis");

            materials.raw.insert(
                raw["name"].to_string(),
                Material {
                    name: raw["name"].to_string(),
                    name_localised: raw["name_localised"].to_string(),
                    grade: raw["grade"].as_u64().unwrap(),
                    count: 0,
                    maximum: raw["maximum"].as_u64().unwrap(),
                    category: raw["category"].to_string(),
                    locations,
                    sources,
                    engineering,
                    synthesis,
                    description: raw["description"].to_string(),
                },
            );
        }

        materials
    }
}

impl Material {
    pub fn get_name(&self) -> String {
        return if self.name_localised != "null" {
            self.name_localised.clone()
        } else {
            let mut name = self.name.clone();
            let char = self
                .name
                .clone()
                .chars()
                .next()
                .unwrap()
                .to_uppercase()
                .to_string();
            name.replace_range(0..1, char.as_str());
            name
        };
    }
}

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub name_localised: String,
    pub grade: u64,
    pub count: u64,
    pub maximum: u64,
    pub category: String,
    pub locations: Vec<String>,
    pub sources: Vec<String>,
    pub engineering: Vec<String>,
    pub synthesis: Vec<String>,
    pub description: String,
}

fn get_array_values(material_array: &JsonValue, key: &str) -> Vec<String> {
    let mut key_values: Vec<String> = vec![];
    let key_array = &material_array[key];
    for j in 0..key_array.len() {
        key_values.push(key_array[j].to_string())
    }
    key_values
}
