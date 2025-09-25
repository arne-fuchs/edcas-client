use std::fmt::write;

use log::{error, warn};

pub enum Tables {
    Government,
    EconomyType,
    Security,
    Allegiance,
    Happiness,
    StationType,
    LandingPadsTypes,
    StationServicesTypes,
    ConflictStatus,
    FactionName,
    FactionStateName,
    WarType,
    Power,
    CommodityName,
    ShipName,
    ModulName,
    //Body stuff
    Volcanism,
    Atmosphere,
    PlanetClass,
    AtmosphereType,
    TerraformState,
    PlanetCompositionType,
    MaterialType,
    RingClass,
    StarType,
}
impl std::fmt::Display for Tables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tables::Government => write!(f, "government"),
            Tables::EconomyType => write!(f, "economy_type"),
            Tables::Security => write!(f, "security"),
            Tables::Allegiance => write!(f, "allegiance"),
            Tables::Happiness => write!(f, "happiness"),
            Tables::StationType => write!(f, "station_type"),
            Tables::LandingPadsTypes => write!(f, "landing_pads_types"),
            Tables::StationServicesTypes => write!(f, "station_services_types"),
            Tables::WarType => write!(f, "war_type"),
            Tables::FactionName => write!(f, "faction_name"),
            Tables::FactionStateName => write!(f, "faction_state_name"),
            Tables::ConflictStatus => write!(f, "conflict_status"),
            Tables::Power => write!(f, "power"),
            Tables::CommodityName => write!(f, "commodity_name"),
            Tables::ShipName => write!(f, "ship_name"),
            Tables::ModulName => write!(f, "modul_name"),
            Tables::Volcanism => write!(f, "volcanism"),
            Tables::Atmosphere => write!(f, "atmosphere"),
            Tables::PlanetClass => write!(f, "planet_class"),
            Tables::AtmosphereType => write!(f, "atmosphere_type"),
            Tables::TerraformState => write!(f, "terraform_state"),
            Tables::PlanetCompositionType => write!(f, "planet_composition_type"),
            Tables::MaterialType => write!(f, "material_type"),
            Tables::RingClass => write!(f, "ring_class"),
            Tables::StarType => write!(f, "star_type"),
        }
    }
}

pub fn value_table(
    table: Tables,
    value: String,
    journal_id: i64,
    client: &mut postgres::Transaction,
) -> Result<i32, postgres::Error> {
    let sql = format!("SELECT id FROM {} WHERE value=$1", table);
    let id: Option<i32> = match client.query_one(sql.as_str(), &[&value]) {
        Ok(row) => {
            if row.is_empty() {
                None
            } else {
                Some(row.get(0))
            }
        }
        Err(err) => {
            warn!(
                "[{}]value_table {}: Unable to execute sql statement for value {}: {}",
                journal_id, table, value, err
            );
            None
        }
    };
    let sql = format!(
        "INSERT INTO {} (value,journal_id) VALUES ($1,$2) RETURNING id",
        table
    );
    match id {
        None => {
            match client.query_one(sql.as_str(), &[&value, &journal_id]) {
                Ok(row) => Ok(row.get(0)),
                Err(err) => {
                    error!("[{}]value_table {}: Unable to execute sql insert statement for value {}: {}",journal_id,table,value,err);
                    Err(err)
                }
            }
        }
        Some(id) => Ok(id),
    }
}
