use json::JsonValue;
use log::{error, info, warn};
use postgres::Client;

use crate::eddn::{edcas_error::EdcasError, faction::{self, insert_station_factions}, star_system};

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
}
impl Tables {
    fn to_string(&self) -> String {
        match self {
            Tables::Government => "government".to_string(),
            Tables::EconomyType => "economy_type".to_string(),
            Tables::Security => "security".to_string(),
            Tables::Allegiance => "allegiance".to_string(),
            Tables::Happiness => "happiness".to_string(),
            Tables::StationType => "station_type".to_string(),
            Tables::LandingPadsTypes => "landing_pads_types".to_string(),
            Tables::StationServicesTypes => "station_services_types".to_string(),
            Tables::WarType => "war_type".to_string(),
            Tables::FactionName => "faction_name".to_string(),
            Tables::FactionStateName => "faction_state_name".to_string(),
            Tables::ConflictStatus => "conflict_status".to_string(),
            Tables::Power => "power".to_string(),
            Tables::CommodityName => "commodity_name".to_string(),
            Tables::ShipName => "ship_name".to_string(),
            Tables::ModulName => "modul_name".to_string(),
            Tables::Volcanism => "volcanism".to_string(),
            Tables::Atmosphere => "atmosphere".to_string(),
            Tables::PlanetClass => "planet_class".to_string(),
            Tables::AtmosphereType => "atmosphere_type".to_string(),
            Tables::TerraformState => "terraform_state".to_string(),
            Tables::PlanetCompositionType => "planet_composition_type".to_string(),
            Tables::MaterialType => "material_type".to_string(),
        }
    }
}
pub fn value_table(
    table: Tables,
    value: String,
    journal_id: i64,
    client: &mut Client,
) -> Result<i32, postgres::Error> {
    let sql = format!("SELECT id FROM {} WHERE value=$1", table.to_string());
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
                journal_id,
                table.to_string(),
                value,
                err
            );
            None
        }
    };
    let sql = format!(
        "INSERT INTO {} (value,journal_id) VALUES ($1,$2) RETURNING id",
        table.to_string()
    );
    match id {
        None => {
            match client.query_one(sql.as_str(), &[&value, &journal_id]) {
                Ok(row) => Ok(row.get(0)),
                Err(err) => {
                    error!("[{}]value_table {}: Unable to execute sql insert statement for value {}: {}",journal_id,table.to_string(),value,err);
                    Err(err)
                }
            }
        }
        Some(id) => Ok(id),
    }
}

pub fn interpret_json(journal_id: i64, event: &str, json: JsonValue, client: &mut Client) -> Result<(), EdcasError> {
    match event {
        //Navigation
        //{ "timestamp":"2022-10-16T23:25:31Z", "event":"FSDJump", "Taxi":false, "Multicrew":false, "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarPos":[-9534.00000,-905.28125,19802.03125], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_HighTech;", "SystemEconomy_Localised":"Hightech", "SystemSecondEconomy":"$economy_Military;", "SystemSecondEconomy_Localised":"Militär", "SystemGovernment":"$government_Confederacy;", "SystemGovernment_Localised":"Konföderation", "SystemSecurity":"$SYSTEM_SECURITY_medium;", "SystemSecurity_Localised":"Mittlere Sicherheit", "Population":151752, "Body":"Ogmar A", "BodyID":1, "BodyType":"Star", "JumpDist":8.625, "FuelUsed":0.024493, "FuelLevel":31.975506, "Factions":[ { "Name":"Jaques", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "PendingStates":[ { "State":"Outbreak", "Trend":0 } ], "ActiveStates":[ { "State":"Election" } ] }, { "Name":"ICU Colonial Corps", "FactionState":"War", "Government":"Communism", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":96.402496, "PendingStates":[ { "State":"Expansion", "Trend":0 } ], "ActiveStates":[ { "State":"War" } ] }, { "Name":"Societas Eruditorum de Civitas Dei", "FactionState":"War", "Government":"Dictatorship", "Influence":0.119192, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":46.414799, "ActiveStates":[ { "State":"War" } ] }, { "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom", "Government":"Confederacy", "Influence":0.406061, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-75.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"Likedeeler of Colonia", "FactionState":"None", "Government":"Democracy", "Influence":0.068687, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.002500 }, { "Name":"Colonia Tech Combine", "FactionState":"Election", "Government":"Cooperative", "Influence":0.138384, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":4.850000, "ActiveStates":[ { "State":"Election" } ] }, { "Name":"Milanov's Reavers", "FactionState":"Bust", "Government":"Anarchy", "Influence":0.010101, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":0.000000, "RecoveringStates":[ { "State":"Terrorism", "Trend":0 } ], "ActiveStates":[ { "State":"Bust" } ] } ], "SystemFaction":{ "Name":"GalCop Colonial Defence Commission", "FactionState":"Boom" }, "Conflicts":[ { "WarType":"election", "Status":"active", "Faction1":{ "Name":"Jaques", "Stake":"Guerrero Military Base", "WonDays":1 }, "Faction2":{ "Name":"Colonia Tech Combine", "Stake":"", "WonDays":0 } }, { "WarType":"war", "Status":"active", "Faction1":{ "Name":"ICU Colonial Corps", "Stake":"Boulaid Command Facility", "WonDays":1 }, "Faction2":{ "Name":"Societas Eruditorum de Civitas Dei", "Stake":"Chatterjee's Respite", "WonDays":0 } } ] }
        "FSDJump" => {
            let allegiance = value_table(
                Tables::Allegiance,
                json["SystemAllegiance"].to_string(),
                journal_id,
                client,
            )?;
            let economy = value_table(
                Tables::EconomyType,
                json["SystemEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let second_economy = value_table(
                Tables::EconomyType,
                json["SystemSecondEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let government = value_table(
                Tables::Government,
                json["SystemGovernment"].to_string(),
                journal_id,
                client,
            )?;
            let security = value_table(
                Tables::Security,
                json["SystemSecurity"].to_string(),
                journal_id,
                client,
            )?;
            if json.has_key("Powers") {
                let power_size = json.len();
                for i in 0..power_size {
                    value_table(
                        Tables::Power,
                        json["Powers"][i].to_string(),
                        journal_id,
                        client,
                    )?;
                }
            }
            let controlling_power = if json.has_key("ControllingPower") {
                Some(value_table(
                    Tables::Power,
                    json["ControllingPower"].to_string(),
                    journal_id,
                    client,
                )?)
            } else {
                None
            };

            let system_address = star_system::insert_star_system(
                json["SystemAddress"]
                    .as_i64()
                    .ok_or(format!("[{}] No SystemAddress in json", journal_id))?,
                json["StarSystem"].to_string(),
                (
                    json["StarPos"][0]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos x in json", journal_id))?,
                    json["StarPos"][1]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos y in json", journal_id))?,
                    json["StarPos"][2]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos z in json", journal_id))?,
                ),
                allegiance,
                economy,
                second_economy,
                government,
                security,
                json["Population"]
                    .as_i64()
                    .ok_or(format!("[{}] No Population in json", journal_id))?,
                controlling_power,
                journal_id,
                client,
            )?;

            faction::insert_factions(&json, client, &system_address, journal_id)?;
            if json.has_key("Conflicts") {
                faction::insert_conflict(&json, client, &system_address, journal_id)?;
            }

            return Ok(());
        }
        "Location" => {
            //{"Body":"Pru Euq LW-E d11-90 A 5 a","BodyID":14,"BodyType":"Planet","DistFromStarLS":363.522609,"Docked":true,"Factions":[{"ActiveStates":[{"State":"Expansion"}],"Allegiance":"Independent","FactionState":"Expansion","Government":"Confederacy","Happiness":"$Faction_HappinessBand2;","Influence":0.03992,"Name":"HIP 96273 Alliance"},{"ActiveStates":[{"State":"Expansion"}],"Allegiance":"Independent","FactionState":"Expansion","Government":"Feudal","Happiness":"$Faction_HappinessBand2;","Influence":0.873253,"Name":"The Dukes of Mikunn"},{"Allegiance":"Independent","FactionState":"None","Government":"Dictatorship","Happiness":"$Faction_HappinessBand2;","Influence":0.086826,"Name":"The Mercs of Mikunn","PendingStates":[{"State":"Expansion","Trend":0}]}],"MarketID":3706516224,"Multicrew":false,"Population":68585642,"StarPos":[-76.34375,11.3125,1232.25],"StarSystem":"Pru Euq LW-E d11-90","StationEconomies":[{"Name":"$economy_Carrier;","Proportion":1.0}],"StationEconomy":"$economy_Carrier;","StationFaction":{"Name":"FleetCarrier"},"StationGovernment":"$government_Carrier;","StationName":"G4W-21Z","StationServices":["dock","autodock","commodities","contacts","crewlounge","rearm","refuel","repair","engineer","flightcontroller","stationoperations","stationMenu","carriermanagement","carrierfuel","socialspace","bartender"],"StationType":"FleetCarrier","SystemAddress":3102837049827,"SystemAllegiance":"Independent","SystemEconomy":"$economy_Refinery;","SystemFaction":{"FactionState":"Expansion","Name":"The Dukes of Mikunn"},"SystemGovernment":"$government_Feudal;","SystemSecondEconomy":"$economy_Industrial;","SystemSecurity":"$SYSTEM_SECURITY_high;","Taxi":false,"event":"Location","horizons":true,"odyssey":true,"timestamp":"2025-09-07T07:18:42Z"}
            if json.has_key("Powers") {
                let power_size = json.len();
                for i in 0..power_size {
                    value_table(
                        Tables::Power,
                        json["Powers"][i].to_string(),
                        journal_id,
                        client,
                    )?;
                }
            }
            let system_allegiance = value_table(
                Tables::Allegiance,
                json["SystemAllegiance"].to_string(),
                journal_id,
                client,
            )?;
            let controlling_power = if json.has_key("ControllingPower") {
                Some(value_table(
                    Tables::Power,
                    json["ControllingPower"].to_string(),
                    journal_id,
                    client,
                )?)
            } else {
                None
            };
            let system_address = star_system::insert_star_system(
                json["SystemAddress"]
                    .as_i64()
                    .ok_or(format!("[{}]No SystemAddress in json", journal_id))?,
                json["StarSystem"].to_string(),
                (
                    json["StarPos"][0]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos x in json", journal_id))?,
                    json["StarPos"][1]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos y in json", journal_id))?,
                    json["StarPos"][2]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos z in json", journal_id))?,
                ),
                system_allegiance,
                value_table(
                    Tables::EconomyType,
                    json["SystemEconomy"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::EconomyType,
                    json["SystemSecondEconomy"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::Government,
                    json["SystemGovernment"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::Security,
                    json["SystemSecurity"].to_string(),
                    journal_id,
                    client,
                )?,
                json["Population"].as_i64().unwrap_or_default(),
                controlling_power,
                journal_id,
                client,
            )?;

            faction::insert_factions(&json, client, &system_address, journal_id)?;
            if json.has_key("Conflicts") {
                faction::insert_conflict(&json, client, &system_address, journal_id)?;
            }

            if json["Docked"]
                .as_bool()
                .ok_or(format!("[{}] No Docked in json", journal_id))?
            {
                //Case: docked
                //{ "timestamp":"2022-10-16T20:54:45Z", "event":"Location", "DistFromStarLS":1007.705243, "Docked":true, "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "StationFaction":{ "Name":"FleetCarrier" }, "StationGovernment":"$government_Carrier;", "StationGovernment_Localised":"Privateigentum", "StationServices":[ "dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "carriermanagement", "carrierfuel", "livery", "voucherredemption", "socialspace", "bartender", "vistagenomics" ], "StationEconomy":"$economy_Carrier;", "StationEconomy_Localised":"Privatunternehmen", "StationEconomies":[ { "Name":"$economy_Carrier;", "Name_Localised":"Privatunternehmen", "Proportion":1.000000 } ], "Taxi":false, "Multicrew":false, "StarSystem":"Colonia", "SystemAddress":3238296097059, "StarPos":[-9530.50000,-910.28125,19808.12500], "SystemAllegiance":"Independent", "SystemEconomy":"$economy_Tourism;", "SystemEconomy_Localised":"Tourismus", "SystemSecondEconomy":"$economy_HighTech;", "SystemSecondEconomy_Localised":"Hightech", "SystemGovernment":"$government_Cooperative;", "SystemGovernment_Localised":"Kooperative", "SystemSecurity":"$SYSTEM_SECURITY_low;", "SystemSecurity_Localised":"Geringe Sicherheit", "Population":583869, "Body":"Colonia 2 c", "BodyID":18, "BodyType":"Planet", "Factions":[ { "Name":"Jaques", "FactionState":"Investment", "Government":"Cooperative", "Influence":0.454092, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand1;", "Happiness_Localised":"In Hochstimmung", "MyReputation":100.000000, "RecoveringStates":[ { "State":"PublicHoliday", "Trend":0 } ], "ActiveStates":[ { "State":"Investment" }, { "State":"CivilLiberty" } ] }, { "Name":"Colonia Council", "FactionState":"Boom", "Government":"Cooperative", "Influence":0.331337, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":100.000000, "ActiveStates":[ { "State":"Boom" } ] }, { "Name":"People of Colonia", "FactionState":"None", "Government":"Cooperative", "Influence":0.090818, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":27.956400 }, { "Name":"Holloway Bioscience Institute", "FactionState":"None", "Government":"Corporate", "Influence":0.123752, "Allegiance":"Independent", "Happiness":"$Faction_HappinessBand2;", "Happiness_Localised":"Glücklich", "MyReputation":-9.420000, "RecoveringStates":[ { "State":"PirateAttack", "Trend":0 } ] } ], "SystemFaction":{ "Name":"Jaques", "FactionState":"Investment" } }
                let faction_name = value_table(
                    Tables::FactionName,
                    json["StationFaction"]["Name"].to_string(),
                    journal_id,
                    client,
                )?;
                let government = value_table(
                    Tables::Government,
                    json["StationGovernment"].to_string(),
                    journal_id,
                    client,
                )?;
                let economy = value_table(
                    Tables::EconomyType,
                    json["StationEconomy"].to_string(),
                    journal_id,
                    client,
                )?;
                let market_id = json["MarketID"]
                    .as_i64()
                    .ok_or(format!("[{}] No MarketID in json", journal_id))?;
                let station_name = json["StationName"].to_string();
                let station_type = value_table(
                    Tables::StationType,
                    json["StationType"].to_string(),
                    journal_id,
                    client,
                )?;
                insert_station_factions(
                    client,
                    &json,
                    faction_name,
                    government,
                    system_allegiance,
                    &system_address,
                    journal_id,
                )?;

                let market_available = match client.query_one(
                    // language=postgresql
                    "SELECT 1 FROM stations WHERE market_id=$1",
                    &[&market_id],
                ) {
                    Ok(row) => {
                        if row.is_empty() {
                            false
                        } else {
                            true
                        }
                    }
                    Err(err) => {
                        if err.to_string() != "query returned an unexpected number of rows" {
                            error!(
                                "[{}]insert_station: Unable to get station: {}",
                                journal_id, err
                            );
                            return Err(EdcasError::from(err));
                        }
                        false
                    }
                };
                if market_available {
                    //Update
                    match client.execute(
                        // language=postgresql
                        "UPDATE stations
                            SET
                                system_address=$1,
                                name=$2,
                                type=$3,
                                faction_name=$4,
                                government=$5,
                                economy=$6,
                                journal_id=$7
                            WHERE market_id=$8
                                ",
                        &[
                            &system_address,
                            &station_name,
                            &station_type,
                            &faction_name,
                            &government,
                            &economy,
                            &journal_id,
                            &market_id,
                        ],
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!(
                                "[{}]insert_station: Unable to update station: {}",
                                journal_id, err
                            );
                        }
                    }
                } else {
                    //Insert
                    match client.execute(
                        // language=postgresql
                        "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                        &[&market_id,&system_address,&station_name,&station_type,&faction_name,&government,&economy,&journal_id]
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}]insert_station: Unable to insert station: {}",journal_id,err);
                        }
                    }
                }
                match client.execute(
                    // language=postgresql
                    "DELETE FROM station_services WHERE market_id=$1",
                    &[&market_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "[{}]Insert station services: couldn't delete old station service: {}",
                            journal_id, err
                        );
                    }
                }
                let station_services_size = json["StationServices"].len();
                for i in 0..station_services_size {
                    let id = value_table(
                        Tables::StationServicesTypes,
                        json["StationServices"][i].to_string(),
                        journal_id,
                        client,
                    )?;
                    match client.execute(
                        // language=postgresql
                        "INSERT INTO station_services (id, market_id,journal_id) VALUES ($1, $2,$3)",
                        &[&id,&market_id,&journal_id]
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}]Insert station services: couldn't insert station service: {}",journal_id,err);
                        }
                    }
                }
                let economy_size = json["StationEconomies"].len();
                if economy_size > 0 {
                    //Delete all existing facion, since an update comes in
                    match client.execute(
                        // language=postgresql
                        "DELETE FROM station_economies WHERE market_id = $1",
                        &[&market_id],
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] station_economies: Couldn't delete old economy state from station {}: {}",journal_id,market_id,err);
                            return Err(EdcasError::from(err));
                        }
                    }
                }
                for i in 0..economy_size {
                    let json = &json["StationEconomies"][i];
                    let economy = value_table(
                        Tables::EconomyType,
                        json["Name"].to_string(),
                        journal_id,
                        client,
                    )?;
                    let proportion = json["Proportion"].as_f32().unwrap_or_default();
                    match client.execute(
                        // language=postgresql
                        "INSERT INTO station_economies (id, market_id, proportion,journal_id) VALUES ($1, $2, $3,$4)",
                        &[&economy,&market_id,&proportion,&journal_id]
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] station_economies: Couldn't insert station economy: {}",journal_id,err);
                        }
                    }
                }

                if json.has_key("LandingPads") {
                    match client.execute(
                        //language=postgresql
                        "DELETE FROM station_landing_pads WHERE market_id=$1",
                        &[&market_id],
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] landing_pads: Couldn't delete old landing_pads from station {}: {}",journal_id,market_id,err);
                        }
                    }
                    let json = &json["LandingPads"];
                    for entry in json.entries() {
                        let landing_pat_types = value_table(
                            Tables::LandingPadsTypes,
                            entry.0.to_string(),
                            journal_id,
                            client,
                        )?;
                        let count = entry.1.as_i32();
                        match client.execute(
                            // language=postgresql
                            "INSERT INTO station_landing_pads (market_id, landing_pads_type, count, journal_id) VALUES ($1,$2,$3,$4)",
                            &[&market_id,&landing_pat_types,&count,&journal_id]
                        ){
                            Ok(_) => {}
                            Err(err) => {
                                error!("[{}] landing_pads: Couldn't insert landing_pad from station {}: {}",journal_id,market_id,err);
                            }
                        }
                    }
                }
            } else {
                //Horizons players do not have the OnFoot variable
                if json["OnFoot"].as_bool().unwrap_or_default() {
                    //Case: on foot
                    //{"Body":"NLTT 6655 5 a","BodyID":37,"BodyType":"Planet","Conflicts":[{"Faction1":{"Name":"Green Party of NLTT 6655","Stake":"","WonDays":0},"Faction2":{"Name":"NLTT 6655 Blue Travel PLC","Stake":"Rah's Castings","WonDays":0},"Status":"active","WarType":"civilwar"},{"Faction1":{"Name":"Natural NLTT 6655 Constitution Party","Stake":"","WonDays":0},"Faction2":{"Name":"Quebecois Patriots","Stake":"Hartsfield Gateway","WonDays":2},"Status":"active","WarType":"election"}],"ControllingPower":"Pranav Antal","DistFromStarLS":1349.61033,"Docked":false,"Factions":[{"ActiveStates":[{"State":"CivilWar"}],"Allegiance":"Federation","FactionState":"CivilWar","Government":"Democracy","Happiness":"$Faction_HappinessBand2;","Influence":0.09739,"Name":"Green Party of NLTT 6655"},{"Allegiance":"Independent","FactionState":"None","Government":"Dictatorship","Happiness":"$Faction_HappinessBand2;","Influence":0.044177,"Name":"NLTT 6655 Party"},{"ActiveStates":[{"State":"CivilWar"}],"Allegiance":"Independent","FactionState":"CivilWar","Government":"Corporate","Happiness":"$Faction_HappinessBand2;","Influence":0.09739,"Name":"NLTT 6655 Blue Travel PLC"},{"Allegiance":"Independent","FactionState":"None","Government":"Anarchy","Happiness":"$Faction_HappinessBand2;","Influence":0.01004,"Name":"NLTT 6655 Jet Mafia"},{"ActiveStates":[{"State":"Election"}],"Allegiance":"Independent","FactionState":"Election","Government":"Dictatorship","Happiness":"$Faction_HappinessBand2;","Influence":0.095382,"Name":"Natural NLTT 6655 Constitution Party"},{"ActiveStates":[{"State":"Election"}],"Allegiance":"Independent","FactionState":"Election","Government":"Patronage","Happiness":"$Faction_HappinessBand2;","Influence":0.095382,"Name":"Quebecois Patriots","PendingStates":[{"State":"Expansion","Trend":0}]},{"ActiveStates":[{"State":"Expansion"}],"Allegiance":"Independent","FactionState":"Expansion","Government":"Cooperative","Happiness":"$Faction_HappinessBand2;","Influence":0.560241,"Name":"Aseveljet"}],"OnFoot":true,"Population":15123030,"PowerplayState":"Stronghold","PowerplayStateControlProgress":0.353698,"PowerplayStateReinforcement":3314,"PowerplayStateUndermining":26416,"Powers":["Li Yong-Rui","Pranav Antal","Jerome Archer"],"StarPos":[-52.90625,-23.78125,-54.46875],"StarSystem":"NLTT 6655","SystemAddress":2869172381073,"SystemAllegiance":"Independent","SystemEconomy":"$economy_Industrial;","SystemFaction":{"FactionState":"Expansion","Name":"Aseveljet"},"SystemGovernment":"$government_Cooperative;","SystemSecondEconomy":"$economy_Extraction;","SystemSecurity":"$SYSTEM_SECURITY_high;","event":"Location","horizons":true,"odyssey":true,"timestamp":"2025-09-01T12:02:16Z"}
                    //{"Body":"Naren 8 d","BodyID":43,"BodyType":"Planet","ControllingPower":"Li Yong-Rui","DistFromStarLS":3051.834041,"Docked":false,"Factions":[{"Allegiance":"Independent","FactionState":"None","Government":"Democracy","Happiness":"$Faction_HappinessBand2;","Influence":0.01003,"Name":"Workers of Naren Labour"},{"Allegiance":"Federation","FactionState":"None","Government":"Corporate","Happiness":"$Faction_HappinessBand2;","Influence":0.01003,"Name":"Naren Industry"},{"Allegiance":"Independent","FactionState":"None","Government":"Communism","Happiness":"$Faction_HappinessBand2;","Influence":0.934804,"Name":"Nagii Union","PendingStates":[{"State":"Expansion","Trend":0}]},{"Allegiance":"Independent","FactionState":"None","Government":"Dictatorship","Happiness":"$Faction_HappinessBand2;","Influence":0.01003,"Name":"Autocracy of Naren"},{"ActiveStates":[{"State":"CivilUnrest"}],"Allegiance":"Independent","FactionState":"CivilUnrest","Government":"Corporate","Happiness":"$Faction_HappinessBand2;","Influence":0.015045,"Name":"Naren Ltd"},{"Allegiance":"Independent","FactionState":"None","Government":"Anarchy","Happiness":"$Faction_HappinessBand2;","Influence":0.01003,"Name":"Naren Gold Drug Empire"},{"Allegiance":"Independent","FactionState":"None","Government":"Confederacy","Happiness":"$Faction_HappinessBand2;","Influence":0.01003,"Name":"Contagion Confederation"}],"OnFoot":true,"Population":8082516412,"PowerplayState":"Fortified","PowerplayStateControlProgress":0.268283,"PowerplayStateReinforcement":638,"PowerplayStateUndermining":3854,"Powers":["Li Yong-Rui","Pranav Antal","Jerome Archer"],"StarPos":[-47.46875,-18.78125,-114.28125],"StarSystem":"Naren","SystemAddress":3893127809371,"SystemAllegiance":"Independent","SystemEconomy":"$economy_Agri;","SystemFaction":{"Name":"Nagii Union"},"SystemGovernment":"$government_Communism;","SystemSecondEconomy":"$economy_Extraction;","SystemSecurity":"$SYSTEM_SECURITY_high;","event":"Location","horizons":true,"odyssey":true,"timestamp":"2025-09-07T07:21:32Z"}
                    //Currently no additional stuff
                } else {
                    //Case not docked, and not on foot
                    //Currently no additional stuff
                    //{"Body":"Deciat 6","BodyID":24,"BodyType":"Planet","ControllingPower":"A. Lavigny-Duval","DistFromStarLS":2041.05034,"Docked":false,"Factions":[{"Allegiance":"Federation","FactionState":"None","Government":"Democracy","Happiness":"$Faction_HappinessBand2;","Influence":0.163802,"Name":"Independent Deciat Green Party"},{"Allegiance":"Federation","FactionState":"None","Government":"Corporate","Happiness":"$Faction_HappinessBand2;","Influence":0.070779,"Name":"Kremata Incorporated","RecoveringStates":[{"State":"Outbreak","Trend":0}]},{"ActiveStates":[{"State":"Boom"},{"State":"CivilLiberty"}],"Allegiance":"Federation","FactionState":"Boom","Government":"Corporate","Happiness":"$Faction_HappinessBand1;","Influence":0.116279,"Name":"Windri & Co"},{"Allegiance":"Independent","FactionState":"None","Government":"Dictatorship","Happiness":"$Faction_HappinessBand2;","Influence":0.047523,"Name":"Deciat Flag"},{"Allegiance":"Independent","FactionState":"None","Government":"Corporate","Happiness":"$Faction_HappinessBand2;","Influence":0.053589,"Name":"Deciat Corp.","RecoveringStates":[{"State":"InfrastructureFailure","Trend":0}]},{"Allegiance":"Independent","FactionState":"None","Government":"Anarchy","Happiness":"$Faction_HappinessBand2;","Influence":0.012133,"Name":"Deciat Blue Dragons"},{"ActiveStates":[{"State":"Boom"}],"Allegiance":"Independent","FactionState":"Boom","Government":"Feudal","Happiness":"$Faction_HappinessBand2;","Influence":0.535895,"Name":"Ryders of the Void","RecoveringStates":[{"State":"Expansion","Trend":0}]}],"Multicrew":false,"Population":31778844,"PowerplayState":"Stronghold","PowerplayStateControlProgress":0.507154,"PowerplayStateReinforcement":36205,"PowerplayStateUndermining":60251,"Powers":["A. Lavigny-Duval","Zemina Torval"],"StarPos":[122.625,-0.8125,-47.28125],"StarSystem":"Deciat","SystemAddress":6681123623626,"SystemAllegiance":"Independent","SystemEconomy":"$economy_Industrial;","SystemFaction":{"FactionState":"Boom","Name":"Ryders of the Void"},"SystemGovernment":"$government_Feudal;","SystemSecondEconomy":"$economy_Refinery;","SystemSecurity":"$SYSTEM_SECURITY_high;","Taxi":false,"event":"Location","horizons":true,"odyssey":true,"timestamp":"2025-09-07T07:36:00Z"}
                }
            }

            return Ok(());
        }
        "CarrierJump" => {
            //{"Body":"Synuefuae HS-G b15-0","BodyID":0,"BodyType":"Star","Docked":true,"MarketID":3711916288,"Multicrew":false,"Population":0,"StarPos":[5278.9375,-364.1875,-744.125],"StarSystem":"Synuefuae HS-G b15-0","StationEconomies":[{"Name":"$economy_Carrier;","Proportion":1.0}],"StationEconomy":"$economy_Carrier;","StationFaction":{"Name":"FleetCarrier"},"StationGovernment":"$government_Carrier;","StationName":"V9Q-WVZ","StationServices":["dock","autodock","commodities","contacts","exploration","crewlounge","rearm","refuel","repair","engineer","flightcontroller","stationoperations","stationMenu","carriermanagement","carrierfuel","socialspace"],"StationType":"FleetCarrier","SystemAddress":741820277889,"SystemAllegiance":"","SystemEconomy":"$economy_None;","SystemGovernment":"$government_None;","SystemSecondEconomy":"$economy_None;","SystemSecurity":"$GAlAXY_MAP_INFO_state_anarchy;","Taxi":false,"event":"CarrierJump","horizons":true,"odyssey":true,"timestamp":"2025-09-08T16:12:56Z"}

            //It can be, that no market id or station name is being registered:
            //{"Body": "Col 285 Sector CS-V b18-2", "event": "CarrierJump", "BodyID": 0, "Docked": false, "OnFoot": true, "StarPos": [154.0, 247.5, 51.4375], "odyssey": true, "BodyType": "Star", "horizons": true, "timestamp": "2025-09-08T16:25:01Z", "Population": 0, "StarSystem": "Col 285 Sector CS-V b18-2", "SystemAddress": 5070880843193, "SystemEconomy": "$economy_None;", "SystemSecurity": "$GAlAXY_MAP_INFO_state_anarchy;", "SystemAllegiance": "", "SystemGovernment": "$government_None;", "SystemSecondEconomy": "$economy_None;"}
            if !json.has_key("MarketID") {
                //In this case, just abort
                return Ok(());
                //Happens on foot
            }

            let faction_name = value_table(
                Tables::FactionName,
                json["StationFaction"]["Name"].to_string(),
                journal_id,
                client,
            )?;
            let government = value_table(
                Tables::Government,
                json["StationGovernment"].to_string(),
                journal_id,
                client,
            )?;
            let station_economy = value_table(
                Tables::EconomyType,
                json["StationEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let distance = json["DistFromStarLS"].as_f32().unwrap_or_default();
            let market_id = json["MarketID"]
                .as_i64()
                .ok_or(format!("[{}] No MarketID in json", journal_id))?;
            let station_name = json["StationName"].to_string();
            let station_type = value_table(
                Tables::StationType,
                json["StationType"].to_string(),
                journal_id,
                client,
            )?;

            let system_allegiance = value_table(
                Tables::Allegiance,
                json["SystemAllegiance"].to_string(),
                journal_id,
                client,
            )?;
            let economy = value_table(
                Tables::EconomyType,
                json["SystemEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let second_economy = value_table(
                Tables::EconomyType,
                json["SystemSecondEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let security = value_table(
                Tables::Security,
                json["SystemSecurity"].to_string(),
                journal_id,
                client,
            )?;
            if json.has_key("Powers") {
                let power_size = json.len();
                for i in 0..power_size {
                    value_table(
                        Tables::Power,
                        json["Powers"][i].to_string(),
                        journal_id,
                        client,
                    )?;
                }
            }
            let controlling_power = if json.has_key("ControllingPower") {
                Some(value_table(
                    Tables::Power,
                    json["ControllingPower"].to_string(),
                    journal_id,
                    client,
                )?)
            } else {
                None
            };

            let system_address = star_system::insert_star_system(
                json["SystemAddress"]
                    .as_i64()
                    .ok_or(format!("[{}]No SystemAddress in json", journal_id))?,
                json["StarSystem"].to_string(),
                (
                    json["StarPos"][0]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos x in json", journal_id))?,
                    json["StarPos"][1]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos y in json", journal_id))?,
                    json["StarPos"][2]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos z in json", journal_id))?,
                ),
                system_allegiance,
                economy,
                second_economy,
                government,
                security,
                json["Population"].as_i64().unwrap_or_default(),
                controlling_power,
                journal_id,
                client,
            )?;

            insert_station_factions(
                client,
                &json,
                faction_name,
                government,
                system_allegiance,
                &system_address,
                journal_id,
            )?;

            let market_available = match client.query_one(
                // language=postgresql
                "SELECT 1 FROM stations WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(row) => {
                    if row.is_empty() {
                        false
                    } else {
                        true
                    }
                }
                Err(err) => {
                    if err.to_string() != "query returned an unexpected number of rows" {
                        error!(
                            "[{}] insert_station: Unable to get station: {}",
                            journal_id, err
                        );
                        return Err(EdcasError::from(err));
                    }
                    false
                }
            };
            if market_available {
                //Update
                match client.execute(
                    // language=postgresql
                    "UPDATE stations
                            SET
                                system_address=$1,
                                name=$2,
                                type=$3,
                                faction_name=$4,
                                government=$5,
                                economy=$6,
                                journal_id=$7
                            WHERE market_id=$8
                                ",
                    &[
                        &system_address,
                        &station_name,
                        &station_type,
                        &faction_name,
                        &government,
                        &station_economy,
                        &journal_id,
                        &market_id,
                    ],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "[{}]insert_station: Unable to update station: {}",
                            journal_id, err
                        );
                    }
                }
            } else {
                //Insert
                match client.execute(
                    // language=postgresql
                    "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    &[&market_id,&system_address,&station_name,&station_type,&faction_name,&government,&station_economy,&distance,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}]insert_station: Unable to insert station: {}",journal_id,err);
                    }
                }
            }
            match client.execute(
                // language=postgresql
                "DELETE FROM station_services WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "[{}]Insert station services: couldn't delete old station service: {}",
                        journal_id, err
                    );
                }
            }
            let station_services_size = json["StationServices"].len();
            for i in 0..station_services_size {
                let id = value_table(
                    Tables::StationServicesTypes,
                    json["StationServices"][i].to_string(),
                    journal_id,
                    client,
                )?;
                match client.execute(
                    // language=postgresql
                    "INSERT INTO station_services (id, market_id,journal_id) VALUES ($1, $2,$3)",
                    &[&id, &market_id, &journal_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "[{}] Insert station services: couldn't insert station service: {}",
                            journal_id, err
                        );
                    }
                }
            }
            let economy_size = json["StationEconomies"].len();
            if economy_size > 0 {
                //Delete all existing facion, since an update comes in
                match client.execute(
                    // language=postgresql
                    "DELETE FROM station_economies WHERE market_id = $1",
                    &[&market_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] station_economies: Couldn't delete old economy state from station {}: {}",journal_id,market_id,err);
                        return Err(EdcasError::from(err));
                    }
                }
            }
            for i in 0..economy_size {
                let json = &json["StationEconomies"][i];
                let economy = value_table(
                    Tables::EconomyType,
                    json["Name"].to_string(),
                    journal_id,
                    client,
                )?;
                let proportion = json["Proportion"].as_f32().unwrap_or_default();
                match client.execute(
                    // language=postgresql
                    "INSERT INTO station_economies (id, market_id, proportion,journal_id) VALUES ($1, $2, $3,$4)",
                    &[&economy,&market_id,&proportion,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] station_economies: Couldn't insert station economy: {}",journal_id,err);
                    }
                }
            }
            return Ok(());
        }
        "SupercruiseEntry" => {
            //Probably nothing
            info!("Registered SupercruiseEntry: {}", journal_id);
            return Ok(());
        }
        "SupercruiseExit" => {
            //Probably nothing
            info!("Registered SupercruiseExit: {}", journal_id);
            return Ok(());
        }
        "StartJump" => {
            //{ "timestamp":"2022-10-16T23:25:05Z", "event":"StartJump", "JumpType":"Hyperspace", "StarSystem":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K" }
            //Probably nothing
            info!("Registered StartJump: {}", journal_id);
            return Ok(());
        }
        //{ "timestamp":"2022-10-16T23:24:46Z", "event":"FSDTarget", "Name":"Ogmar", "SystemAddress":84180519395914, "StarClass":"K", "RemainingJumpsInRoute":1 }
        "FSDTarget" => {
            //Probably nothing
            info!("Registered FSDTarget: {}", journal_id);
            return Ok(());
        } //If system has been targeted
        "NavRoute" => {
            //{"Route": [{"StarPos": [18606.21875, -64.09375, 33004.25], "StarClass": "M", "StarSystem": "Spase RA-V b36-27", "SystemAddress": 60294227188025}, {"StarPos": [18611.34375, -67.03125, 33005.25], "StarClass": "G", "StarSystem": "Spase IF-Y c17-4", "SystemAddress": 1214569720986}, {"StarPos": [18614.75, -67.75, 32998.59375], "StarClass": "M", "StarSystem": "Spase IF-Y c17-7", "SystemAddress": 2039203441818}, {"StarPos": [18618.28125, -68.34375, 32994.0], "StarClass": "K", "StarSystem": "Spase JF-Y c17-18", "SystemAddress": 5062927527066}, {"StarPos": [18620.03125, -72.21875, 32992.6875], "StarClass": "M", "StarSystem": "Spase QP-W b35-0", "SystemAddress": 920867658033}, {"StarPos": [18624.5, -73.78125, 32987.9375], "StarClass": "G", "StarSystem": "Spase JF-Y c17-79", "SystemAddress": 21830479850650}, {"StarPos": [18626.6875, -72.78125, 32985.65625], "StarClass": "Y", "StarSystem": "Spase JQ-R a72-0", "SystemAddress": 7367478915688}, {"StarPos": [18629.53125, -75.1875, 32980.625], "StarClass": "K", "StarSystem": "Spase JF-Y c17-88", "SystemAddress": 24304381013146}, {"StarPos": [18630.5625, -73.15625, 32977.0625], "StarClass": "M", "StarSystem": "Spase QP-W b35-28", "SystemAddress": 62493518813489}, {"StarPos": [18635.46875, -73.25, 32976.65625], "StarClass": "L", "StarSystem": "Spase GK-T a71-0", "SystemAddress": 7368552657504}, {"StarPos": [18636.375, -77.34375, 32976.875], "StarClass": "K", "StarSystem": "Spase JF-Y c17-13", "SystemAddress": 3688537992346}, {"StarPos": [18637.25, -80.28125, 32976.1875], "StarClass": "TTS", "StarSystem": "Spase IF-T a71-0", "SystemAddress": 7368552526432}, {"StarPos": [18640.65625, -82.09375, 32969.65625], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-6", "SystemAddress": 14115275626793}, {"StarPos": [18641.53125, -80.6875, 32965.84375], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-26", "SystemAddress": 58095740737833}, {"StarPos": [18641.96875, -81.75, 32961.8125], "StarClass": "M", "StarSystem": "Spase NJ-Y b34-16", "SystemAddress": 36105508182313}, {"StarPos": [18646.6875, -82.09375, 32958.125], "StarClass": "T", "StarSystem": "Spase BT-W a69-1", "SystemAddress": 24961812312656}, {"StarPos": [18646.6875, -83.3125, 32956.5625], "StarClass": "Y", "StarSystem": "Spase BT-W a69-0", "SystemAddress": 7369626268240}, {"StarPos": [18650.84375, -84.21875, 32953.15625], "StarClass": "G", "StarSystem": "Spase FZ-Z c16-88", "SystemAddress": 24304381013138}, {"StarPos": [18653.34375, -81.46875, 32952.90625], "StarClass": "Y", "StarSystem": "Spase XM-Y a68-0", "SystemAddress": 7369626268232}, {"StarPos": [18659.125, -82.875, 32950.25], "StarClass": "K", "StarSystem": "Spase GZ-Z c16-95", "SystemAddress": 26228593470610}, {"StarPos": [18659.28125, -81.40625, 32948.6875], "StarClass": "M", "StarSystem": "Spase KD-A b34-10", "SystemAddress": 22911637084449}, {"StarPos": [18662.5625, -81.53125, 32941.9375], "StarClass": "K", "StarSystem": "Spase GZ-Z c16-65", "SystemAddress": 17982256262290}, {"StarPos": [18664.5, -78.875, 32938.625], "StarClass": "M", "StarSystem": "Spase KD-A b34-26", "SystemAddress": 58096009173281}, {"StarPos": [18669.5, -80.25, 32933.75], "StarClass": "M", "StarSystem": "Spase GX-B b33-11", "SystemAddress": 25110660339993}, {"StarPos": [18669.5625, -78.9375, 32930.71875], "StarClass": "K", "StarSystem": "Spase CT-B c16-90", "SystemAddress": 24854203935882}, {"StarPos": [18672.09375, -77.9375, 32924.875], "StarClass": "F", "StarSystem": "Spase MT-O d7-82", "SystemAddress": 2831901775427}, {"StarPos": [18675.21875, -80.3125, 32921.625], "StarClass": "K", "StarSystem": "Spase CT-B c16-0", "SystemAddress": 115192310922}, {"StarPos": [18677.65625, -80.375, 32920.21875], "StarClass": "M", "StarSystem": "Spase HX-B b33-28", "SystemAddress": 62494324119833}, {"StarPos": [18678.0, -78.4375, 32917.28125], "StarClass": "T", "StarSystem": "Spase OU-D a66-1", "SystemAddress": 24965033538096}, {"StarPos": [18682.0, -78.0625, 32917.09375], "StarClass": "M", "StarSystem": "Spase HX-B b33-22", "SystemAddress": 49300184586521}, {"StarPos": [18686.0, -83.875, 32915.53125], "StarClass": "Y", "StarSystem": "Spase PU-D a66-0", "SystemAddress": 7373921235504}, {"StarPos": [18689.6875, -86.125, 32911.75], "StarClass": "M", "StarSystem": "Spase FM-D b32-17", "SystemAddress": 38305068243217}, {"StarPos": [18690.125, -87.9375, 32908.78125], "StarClass": "F", "StarSystem": "Spase MT-O d7-26", "SystemAddress": 907756426819}, {"StarPos": [18691.28125, -85.75, 32905.21875], "StarClass": "M", "StarSystem": "Spase CT-B c16-77", "SystemAddress": 21280791145610}, {"StarPos": [18695.09375, -90.25, 32901.0625], "StarClass": "F", "StarSystem": "Spase MT-O d7-89", "SystemAddress": 3072419944003}, {"StarPos": [18695.53125, -89.75, 32899.1875], "StarClass": "M", "StarSystem": "Spase GM-D b32-12", "SystemAddress": 27310220400913}, {"StarPos": [18697.53125, -94.875, 32895.46875], "StarClass": "TTS", "StarSystem": "Spase KD-H a64-0", "SystemAddress": 7374994846240}, {"StarPos": [18698.75, -97.59375, 32892.5], "StarClass": "K", "StarSystem": "Spase ZM-D c15-61", "SystemAddress": 16882811743362}, {"StarPos": [18701.4375, -98.9375, 32888.21875], "StarClass": "M", "StarSystem": "Spase ZM-D c15-60", "SystemAddress": 16607933836418}, {"StarPos": [18704.875, -100.375, 32885.53125], "StarClass": "K", "StarSystem": "Spase ZM-D c15-57", "SystemAddress": 15783300115586}, {"StarPos": [18705.625, -102.5625, 32883.1875], "StarClass": "M", "StarSystem": "Spase CG-F b31-19", "SystemAddress": 42703383189769}, {"StarPos": [18708.65625, -104.71875, 32880.09375], "StarClass": "G", "StarSystem": "Spase MT-O d7-221", "SystemAddress": 7607905408579}, {"StarPos": [18708.375, -104.9375, 32876.03125], "StarClass": "M", "StarSystem": "Spase CG-F b31-26", "SystemAddress": 58096545978633}, {"StarPos": [18710.28125, -105.125, 32873.71875], "StarClass": "M", "StarSystem": "Spase BI-D c15-44", "SystemAddress": 12209887292546}, {"StarPos": [18710.96875, -105.71875, 32868.03125], "StarClass": "M", "StarSystem": "Spase AV-G b30-5", "SystemAddress": 11917057546497}, {"StarPos": [18712.09375, -109.46875, 32867.15625], "StarClass": "M", "StarSystem": "Spase AV-G b30-6", "SystemAddress": 14116080802049}, {"StarPos": [18715.5, -110.34375, 32863.53125], "StarClass": "M", "StarSystem": "Spase BV-G b30-2", "SystemAddress": 5320256215297}, {"StarPos": [18716.15625, -110.0625, 32862.15625], "StarClass": "M", "StarSystem": "Spase BV-G b30-5", "SystemAddress": 11917325981953}, {"StarPos": [18719.875, -111.21875, 32856.71875], "StarClass": "F", "StarSystem": "Spase OO-O d7-133", "SystemAddress": 4584248415811}, {"StarPos": [18720.1875, -112.8125, 32854.90625], "StarClass": "M", "StarSystem": "Spase XO-I b29-6", "SystemAddress": 14116349237497}, {"StarPos": [18721.28125, -115.6875, 32851.03125], "StarClass": "F", "StarSystem": "Spase KI-Q d6-172", "SystemAddress": 5924278212155}, {"StarPos": [18721.53125, -118.1875, 32851.25], "StarClass": "M", "StarSystem": "Spase XO-I b29-20", "SystemAddress": 44902674815225}, {"StarPos": [18721.71875, -122.5, 32846.0], "StarClass": "Y", "StarSystem": "Spase YJ-P a59-0", "SystemAddress": 7377141936632}, {"StarPos": [18724.03125, -119.3125, 32841.625], "StarClass": "A", "StarSystem": "Spase KI-Q d6-114", "SystemAddress": 3931413386811}, {"StarPos": [18728.4375, -119.34375, 32838.875], "StarClass": "M", "StarSystem": "Spase XO-I b29-17", "SystemAddress": 38305605048569}, {"StarPos": [18732.5625, -119.25, 32833.96875], "StarClass": "M", "StarSystem": "Spase TI-K b28-13", "SystemAddress": 29509512026353}, {"StarPos": [18733.03125, -119.96875, 32831.15625], "StarClass": "F", "StarSystem": "Spase KI-Q d6-207", "SystemAddress": 7126869055035}, {"StarPos": [18735.28125, -122.3125, 32827.40625], "StarClass": "Y", "StarSystem": "Spase SX-S a57-0", "SystemAddress": 7379289420264}, {"StarPos": [18735.28125, -124.0625, 32825.09375], "StarClass": "T", "StarSystem": "Spase SX-S a57-1", "SystemAddress": 24971475464680}, {"StarPos": [18739.125, -126.875, 32825.84375], "StarClass": "K", "StarSystem": "Spase YB-F c14-50", "SystemAddress": 13859221843066}, {"StarPos": [18740.8125, -127.90625, 32821.8125], "StarClass": "T", "StarSystem": "Spase QM-U a56-0", "SystemAddress": 7379289289184}, {"StarPos": [18742.0625, -128.34375, 32822.3125], "StarClass": "K", "StarSystem": "Spase YB-F c14-69", "SystemAddress": 19081902075002}, {"StarPos": [18747.96875, -131.3125, 32819.1875], "StarClass": "M", "StarSystem": "Spase WD-K b28-9", "SystemAddress": 20713687374065}, {"StarPos": [18753.03125, -129.5625, 32813.84375], "StarClass": "G", "StarSystem": "Spase UV-G c13-27", "SystemAddress": 7537029983346}, {"StarPos": [18753.875, -128.8125, 32811.96875], "StarClass": "F", "StarSystem": "Spase LI-Q d6-25", "SystemAddress": 873413449275}, {"StarPos": [18754.21875, -123.3125, 32808.53125], "StarClass": "G", "StarSystem": "Spase LI-Q d6-266", "SystemAddress": 9154110395963}, {"StarPos": [18756.75, -123.125, 32806.25], "StarClass": "M", "StarSystem": "Spase RC-M b27-20", "SystemAddress": 44903211686121}, {"StarPos": [18757.6875, -123.4375, 32802.96875], "StarClass": "M", "StarSystem": "Spase RC-M b27-16", "SystemAddress": 36107118663913}, {"StarPos": [18761.3125, -122.34375, 32796.25], "StarClass": "T", "StarSystem": "Spase IF-Y a54-0", "SystemAddress": 7381436903888}, {"StarPos": [18764.25, -126.65625, 32793.5625], "StarClass": "M", "StarSystem": "Spase PR-N b26-12", "SystemAddress": 27311025576161}, {"StarPos": [18765.8125, -126.625, 32793.78125], "StarClass": "M", "StarSystem": "Spase PR-N b26-11", "SystemAddress": 25112002320609}, {"StarPos": [18767.71875, -125.625, 32786.71875], "StarClass": "M", "StarSystem": "Spase PR-N b26-2", "SystemAddress": 5320793020641}, {"StarPos": [18771.15625, -125.8125, 32783.40625], "StarClass": "K", "StarSystem": "Spase PR-N b26-1", "SystemAddress": 3121769765089}, {"StarPos": [18774.28125, -121.15625, 32780.0], "StarClass": "F", "StarSystem": "Spase LI-Q d6-229", "SystemAddress": 7882800076347}, {"StarPos": [18779.3125, -119.78125, 32778.875], "StarClass": "K", "StarSystem": "Spase VV-G c13-71", "SystemAddress": 19631724997746}, {"StarPos": [18784.875, -120.90625, 32774.78125], "StarClass": "M", "StarSystem": "Spase KQ-P b25-5", "SystemAddress": 11918131288281}, {"StarPos": [18786.90625, -118.78125, 32772.28125], "StarClass": "M", "StarSystem": "Spase KQ-P b25-4", "SystemAddress": 9719108032729}, {"StarPos": [18790.5625, -118.3125, 32767.25], "StarClass": "M", "StarSystem": "Spase KQ-P b25-20", "SystemAddress": 44903480121561}, {"StarPos": [18793.4375, -117.125, 32764.75], "StarClass": "Y", "StarSystem": "Spase VG-F a51-0", "SystemAddress": 7384658129328}, {"StarPos": [18796.90625, -116.78125, 32761.4375], "StarClass": "K", "StarSystem": "Spase RP-I c12-59", "SystemAddress": 16333190114410}, {"StarPos": [18799.25, -117.1875, 32758.25], "StarClass": "G", "StarSystem": "Spase HC-S d5-187", "SystemAddress": 6439691064883}, {"StarPos": [18798.375, -117.90625, 32753.5625], "StarClass": "M", "StarSystem": "Spase HK-R b24-8", "SystemAddress": 18515469490385}, {"StarPos": [18801.3125, -119.9375, 32749.09375], "StarClass": "M", "StarSystem": "Spase HK-R b24-18", "SystemAddress": 40505702045905}, {"StarPos": [18805.46875, -117.96875, 32744.09375], "StarClass": "M", "StarSystem": "Spase HK-R b24-12", "SystemAddress": 27311562512593}, {"StarPos": [18806.96875, -120.1875, 32741.53125], "StarClass": "M", "StarSystem": "Spase HK-R b24-6", "SystemAddress": 14117422979281}, {"StarPos": [18810.3125, -119.96875, 32739.375], "StarClass": "K", "StarSystem": "Spase RP-I c12-31", "SystemAddress": 8636608719978}, {"StarPos": [18814.6875, -120.59375, 32737.875], "StarClass": "M", "StarSystem": "Spase HK-R b24-3", "SystemAddress": 7520353212625}, {"StarPos": [18817.53125, -121.4375, 32736.84375], "StarClass": "M", "StarSystem": "Spase IK-R b24-1", "SystemAddress": 3122575136977}, {"StarPos": [18820.4375, -119.96875, 32731.65625], "StarClass": "M", "StarSystem": "Spase OJ-K c11-45", "SystemAddress": 12484966526050}, {"StarPos": [18818.96875, -120.75, 32730.21875], "StarClass": "K", "StarSystem": "Spase OJ-K c11-32", "SystemAddress": 8911553735778}, {"StarPos": [18822.5625, -121.28125, 32723.15625], "StarClass": "K", "StarSystem": "Spase OJ-K c11-50", "SystemAddress": 13859356060770}, {"StarPos": [18827.15625, -121.875, 32721.625], "StarClass": "TTS", "StarSystem": "Spase EE-T b23-21", "SystemAddress": 47103040248009}, {"StarPos": [18827.28125, -125.40625, 32720.84375], "StarClass": "F", "StarSystem": "Spase IC-S d5-140", "SystemAddress": 4824800138803}, {"StarPos": [18825.9375, -126.78125, 32715.1875], "StarClass": "A", "StarSystem": "Spase IC-S d5-103", "SystemAddress": 3553489819187}, {"StarPos": [18830.90625, -124.59375, 32710.28125], "StarClass": "A", "StarSystem": "Spase IC-S d5-172", "SystemAddress": 5924311766579}, {"StarPos": [18834.78125, -122.53125, 32708.4375], "StarClass": "M", "StarSystem": "Spase AY-U b22-4", "SystemAddress": 9719644903617}, {"StarPos": [18837.625, -123.53125, 32704.28125], "StarClass": "M", "StarSystem": "Spase BY-U b22-4", "SystemAddress": 9719913339073}, {"StarPos": [18837.75, -124.125, 32699.5], "StarClass": "M", "StarSystem": "Spase BY-U b22-2", "SystemAddress": 5321866827969}, {"StarPos": [18841.875, -126.96875, 32693.28125], "StarClass": "M", "StarSystem": "Spase ZM-W b21-1", "SystemAddress": 3122843506873}, {"StarPos": [18846.21875, -128.03125, 32692.5], "StarClass": "K", "StarSystem": "Spase KD-M c10-65", "SystemAddress": 17982524664922}, {"StarPos": [18846.21875, -129.28125, 32692.0], "StarClass": "M", "StarSystem": "Spase ZM-W b21-7", "SystemAddress": 16316983040185}, {"StarPos": [18850.15625, -131.40625, 32691.0625], "StarClass": "F", "StarSystem": "Spase EW-T d4-143", "SystemAddress": 4927879353899}, {"StarPos": [18849.71875, -136.5, 32689.84375], "StarClass": "K", "StarSystem": "Spase KD-M c10-33", "SystemAddress": 9186431642714}, {"StarPos": [18850.5625, -141.15625, 32686.875], "StarClass": "G", "StarSystem": "Spase EW-T d4-15", "SystemAddress": 529832842795}, {"StarPos": [18850.40625, -141.1875, 32682.78125], "StarClass": "M", "StarSystem": "Spase ZM-W b21-15", "SystemAddress": 33909169084601}, {"StarPos": [18850.875, -141.59375, 32679.65625], "StarClass": "M", "StarSystem": "Spase ZM-W b21-12", "SystemAddress": 27312099317945}, {"StarPos": [18850.1875, -143.0625, 32676.53125], "StarClass": "F", "StarSystem": "Spase EW-T d4-19", "SystemAddress": 667271796267}, {"StarPos": [18849.875, -139.0, 32672.59375], "StarClass": "M", "StarSystem": "Spase VG-Y b20-5", "SystemAddress": 11918936529073}, {"StarPos": [18854.0625, -139.0625, 32666.125], "StarClass": "M", "StarSystem": "Spase VG-Y b20-3", "SystemAddress": 7520890017969}, {"StarPos": [18857.40625, -137.46875, 32660.3125], "StarClass": "M", "StarSystem": "Spase WG-Y b20-2", "SystemAddress": 5322135197873}, {"StarPos": [18858.34375, -135.5625, 32655.5], "StarClass": "M", "StarSystem": "Spase WG-Y b20-0", "SystemAddress": 924088686769}, {"StarPos": [18862.8125, -139.78125, 32652.71875], "StarClass": "K", "StarSystem": "Spase HX-N c9-41", "SystemAddress": 11385522007122}, {"StarPos": [18864.0625, -142.46875, 32647.03125], "StarClass": "M", "StarSystem": "Spase SA-A b20-10", "SystemAddress": 22914321242281}, {"StarPos": [18868.40625, -141.90625, 32643.625], "StarClass": "M", "StarSystem": "Spase SA-A b20-11", "SystemAddress": 25113344497833}, {"StarPos": [18870.21875, -140.8125, 32639.15625], "StarClass": "M", "StarSystem": "Spase SA-A b20-12", "SystemAddress": 27312367753385}, {"StarPos": [18874.34375, -141.96875, 32635.90625], "StarClass": "K", "StarSystem": "Spase HX-N c9-77", "SystemAddress": 21281126657106}, {"StarPos": [18875.3125, -141.84375, 32635.4375], "StarClass": "M", "StarSystem": "Spase TA-A b20-2", "SystemAddress": 5322403633321}, {"StarPos": [18877.375, -140.875, 32636.34375], "StarClass": "K", "StarSystem": "Spase HX-N c9-2", "SystemAddress": 665283636306}, {"StarPos": [18880.9375, -141.0625, 32635.5625], "StarClass": "M", "StarSystem": "Spase TA-A b20-0", "SystemAddress": 924357122217}, {"StarPos": [18882.25, -141.6875, 32633.71875], "StarClass": "K", "StarSystem": "Spase HX-N c9-71", "SystemAddress": 19631859215442}, {"StarPos": [18885.28125, -140.6875, 32630.28125], "StarClass": "M", "StarSystem": "Spase PU-B b19-5", "SystemAddress": 11919473399969}, {"StarPos": [18885.78125, -138.53125, 32629.625], "StarClass": "M", "StarSystem": "Spase PU-B b19-4", "SystemAddress": 9720450144417}, {"StarPos": [18885.15625, -137.25, 32626.625], "StarClass": "A", "StarSystem": "Spase EW-T d4-68", "SystemAddress": 2350898976299}, {"StarPos": [18885.09375, -136.0625, 32621.8125], "StarClass": "M", "StarSystem": "Spase PU-B b19-11", "SystemAddress": 25113612933281}, {"StarPos": [18888.96875, -137.34375, 32619.4375], "StarClass": "M", "StarSystem": "Spase PU-B b19-15", "SystemAddress": 33909705955489}, {"StarPos": [18895.875, -134.5625, 32616.34375], "StarClass": "T", "StarSystem": "Spase EU-D a38-0", "SystemAddress": 7396469158208}, {"StarPos": [18897.84375, -137.28125, 32615.21875], "StarClass": "K", "StarSystem": "Spase IX-N c9-2", "SystemAddress": 665350745170}, {"StarPos": [18898.0, -139.59375, 32615.59375], "StarClass": "M", "StarSystem": "Spase QU-B b19-0", "SystemAddress": 924625557665}, {"StarPos": [18900.03125, -140.9375, 32615.25], "StarClass": "L", "StarSystem": "Spase QU-B b19-1", "SystemAddress": 3123648813217}, {"StarPos": [18899.9375, -143.4375, 32609.6875], "StarClass": "M", "StarSystem": "Spase MO-D b18-12", "SystemAddress": 27312904624281}, {"StarPos": [18903.53125, -143.59375, 32601.84375], "StarClass": "K", "StarSystem": "Spase ER-P c8-59", "SystemAddress": 16333391440970}, {"StarPos": [18905.625, -142.78125, 32598.6875], "StarClass": "K", "StarSystem": "Spase ER-P c8-56", "SystemAddress": 15508757720138}, {"StarPos": [18907.3125, -145.0, 32597.90625], "StarClass": "K", "StarSystem": "Spase ER-P c8-58", "SystemAddress": 16058513534026}, {"StarPos": [18906.1875, -147.0, 32594.09375], "StarClass": "K", "StarSystem": "Spase GM-P c8-47", "SystemAddress": 13034856524874}, {"StarPos": [18907.25, -149.25, 32589.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-19", "SystemAddress": 5338275130442}, {"StarPos": [18908.375, -152.5, 32585.0625], "StarClass": "M", "StarSystem": "Spase GM-P c8-21", "SystemAddress": 5888030944330}, {"StarPos": [18911.625, -151.71875, 32582.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-61", "SystemAddress": 16883147222090}, {"StarPos": [18912.84375, -152.6875, 32580.0], "StarClass": "A", "StarSystem": "Spase BQ-V d3-113", "SystemAddress": 3897103980067}, {"StarPos": [18917.0, -153.875, 32579.28125], "StarClass": "M", "StarSystem": "Spase LD-F b17-5", "SystemAddress": 11920010205329}, {"StarPos": [18917.78125, -152.4375, 32578.5], "StarClass": "K", "StarSystem": "Spase GM-P c8-23", "SystemAddress": 6437786758218}, {"StarPos": [18922.21875, -150.96875, 32577.46875], "StarClass": "M", "StarSystem": "Spase LD-F b17-3", "SystemAddress": 7521963694225}, {"StarPos": [18922.0, -147.53125, 32577.25], "StarClass": "M", "StarSystem": "Spase LD-F b17-14", "SystemAddress": 31711219505297}, {"StarPos": [18923.5, -146.40625, 32572.4375], "StarClass": "G", "StarSystem": "Spase CG-R c7-34", "SystemAddress": 9461443734594}, {"StarPos": [18926.28125, -147.78125, 32570.03125], "StarClass": "M", "StarSystem": "Spase HX-G b16-12", "SystemAddress": 27313172994185}, {"StarPos": [18930.21875, -153.15625, 32571.71875], "StarClass": "F", "StarSystem": "Spase BQ-V d3-293", "SystemAddress": 10081856886307}, {"StarPos": [18932.28125, -153.78125, 32565.125], "StarClass": "M", "StarSystem": "Spase HX-G b16-9", "SystemAddress": 20716103227529}, {"StarPos": [18933.15625, -158.15625, 32560.71875], "StarClass": "M", "StarSystem": "Spase HX-G b16-14", "SystemAddress": 31711219505289}, {"StarPos": [18931.375, -157.5, 32558.28125], "StarClass": "M", "StarSystem": "Spase HX-G b16-5", "SystemAddress": 11920010205321}, {"StarPos": [18927.53125, -156.46875, 32555.59375], "StarClass": "M", "StarSystem": "Spase HX-G b16-10", "SystemAddress": 22915126483081}, {"StarPos": [18927.0, -157.34375, 32552.4375], "StarClass": "K", "StarSystem": "Spase CG-R c7-43", "SystemAddress": 11935344897090}, {"StarPos": [18929.09375, -156.25, 32546.8125], "StarClass": "F", "StarSystem": "Spase BQ-V d3-103", "SystemAddress": 3553506596387}, {"StarPos": [18929.65625, -158.53125, 32544.625], "StarClass": "M", "StarSystem": "Spase DR-I b15-6", "SystemAddress": 14119033460865}, {"StarPos": [18929.625, -161.90625, 32542.375], "StarClass": "M", "StarSystem": "Spase DR-I b15-8", "SystemAddress": 18517079971969}, {"StarPos": [18933.34375, -163.5, 32543.15625], "StarClass": "K", "StarSystem": "Spase CG-R c7-50", "SystemAddress": 13859490245698}, {"StarPos": [18939.3125, -161.5625, 32538.875], "StarClass": "K", "StarSystem": "Spase DG-R c7-45", "SystemAddress": 12485167819842}, {"StarPos": [18944.125, -165.46875, 32534.09375], "StarClass": "F", "StarSystem": "Spase XJ-X d2-273", "SystemAddress": 9394662118939}, {"StarPos": [18945.875, -164.96875, 32531.65625], "StarClass": "M", "StarSystem": "Spase AL-K b14-16", "SystemAddress": 36109534451833}, {"StarPos": [18947.4375, -162.28125, 32526.8125], "StarClass": "G", "StarSystem": "Spase ZZ-S c6-1", "SystemAddress": 390539914298}, {"StarPos": [18944.25, -159.46875, 32523.03125], "StarClass": "M", "StarSystem": "Spase AL-K b14-0", "SystemAddress": 925162363001}], "event": "NavRoute", "odyssey": true, "horizons": true, "timestamp": "2025-09-08T16:23:42Z"}
            //TODO What to do with NavRoute?
            return Ok(());
        }
        "NavRouteClear" => {
            info!("Registered FSDTarget: {}", journal_id);
            return Ok(());
        } //If navigation is complete -> no further information

        //Approaching
        "ApproachSettlement" => {
            //{"Name": "Bevis Foundry", "event": "ApproachSettlement", "BodyID": 11, "StarPos": [517.625, 45.5, 3351.65625], "odyssey": false, "BodyName": "Smojai JR-N d6-35 A 3 a", "Latitude": 30.852997, "MarketID": 4284372995, "horizons": true, "Longitude": 148.809814, "timestamp": "2025-09-08T16:23:36Z", "StarSystem": "Smojai JR-N d6-35", "SystemAddress": 1213185657531, "StationEconomy": "$economy_Industrial;", "StationFaction": {"Name": "Bot Network"}, "StationServices": ["dock", "autodock", "commodities", "contacts", "missions", "outfitting", "rearm", "refuel", "repair", "engineer", "facilitator", "flightcontroller", "stationoperations", "powerplay", "searchrescue", "stationMenu", "shop", "livery", "socialspace", "registeringcolonisation"], "StationEconomies": [{"Name": "$economy_Industrial;", "Proportion": 1.7}, {"Name": "$economy_Agri;", "Proportion": 0.2}, {"Name": "$economy_Refinery;", "Proportion": 0.2}, {"Name": "$economy_Extraction;", "Proportion": 0.05}], "StationAllegiance": "Federation", "StationGovernment": "$government_Corporate;"}
            //TODO Implement
        }
        "ApproachBody" => {
            info!("Registered ApproachBody: {}", journal_id);
            return Ok(());
        }
        "LeaveBody" => {
            info!("Registered LeaveBody: {}", journal_id);
            return Ok(());
        }
        "Liftoff" => {
            info!("Registered Liftoff: {}", journal_id);
            return Ok(());
        }
        "Touchdown" => {
            info!("Registered Touchdown: {}", journal_id);
            return Ok(());
        }
        "Embark" => {
            info!("Registered Embark: {}", journal_id);
            return Ok(());
        }
        "Disembark" => {
            info!("Registered Disembark: {}", journal_id);
            return Ok(());
        }

        //Scanning
        "DiscoveryScan" => {
            info!("Registered DiscoveryScan: {}", journal_id);
            return Ok(());
        }
        "FSSAllBodiesFound" => {
            //{"Count": 36, "event": "FSSAllBodiesFound", "StarPos": [581.65625, 154.8125, -189.8125], "odyssey": true, "horizons": true, "timestamp": "2025-09-08T16:24:12Z", "SystemName": "Wregoe ST-I d9-10", "SystemAddress": 354209007955}
            //TODO Does it make sense to save the body count?
            return Ok(());
        }
        //{ "timestamp":"2022-10-16T23:46:48Z", "event":"FSSDiscoveryScan", "Progress":0.680273, "BodyCount":21, "NonBodyCount":80, "SystemName":"Ogmar", "SystemAddress":84180519395914 }
        "FSSDiscoveryScan" => {
            //TODO Does it make sense to save the body + non-body count?
            return Ok(());
        } //Honk
        "FSSBodySignals" => {
            //language=json
            let _ = r#"{
              "event": "FSSBodySignals",
              "BodyID": 14,
              "Signals": [
                {
                  "Type": "$SAA_SignalType_Biological;",
                  "Count": 3
                }
              ],
              "StarPos": [
                582.4375,
                -19.90625,
                -10102.34375
              ],
              "odyssey": true,
              "BodyName": "Eos Ain QO-Z d13-0 A 5 e",
              "horizons": true,
              "timestamp": "2025-09-08T17:14:20Z",
              "StarSystem": "Eos Ain QO-Z d13-0",
              "SystemAddress": 10611590523
            }"#;
            //TODO Save body signals Implement
        }
        "SAASignalsFound" => {
            //language=json
            let _ = r#"
                    {
                      "event": "SAASignalsFound",
                      "BodyID": 53,
                      "Genuses": [
                        {
                          "Genus": "$Codex_Ent_Bacterial_Genus_Name;"
                        },
                        {
                          "Genus": "$Codex_Ent_Fonticulus_Genus_Name;"
                        }
                      ],
                      "Signals": [
                        {
                          "Type": "$SAA_SignalType_Biological;",
                          "Count": 2
                        }
                      ],
                      "StarPos": [
                        1840.46875,
                        316.09375,
                        2038.84375
                      ],
                      "odyssey": true,
                      "BodyName": "Phylucs BR-H b25-0 B 4 c",
                      "horizons": true,
                      "timestamp": "2025-09-08T17:24:49Z",
                      "StarSystem": "Phylucs BR-H b25-0",
                      "SystemAddress": 695651608793
                    }
            "#;
            //TODO Save body signals Implement
        }
        "FSSSignalDiscovered" => {
            //language=json
            let _ = r#"
            {
              "event": "FSSSignalDiscovered",
              "StarPos": [
                -22.28125,
                -5.8125,
                -28.09375
              ],
              "odyssey": true,
              "signals": [
                {
                  "IsStation": true,
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "Giacconi Sanctuary",
                  "SignalType": "StationCoriolis"
                },
                {
                  "IsStation": true,
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "KILO FOXTROT K7N-0TB",
                  "SignalType": "FleetCarrier"
                },
                {
                  "timestamp": "2025-09-08T17:49:26Z",
                  "SignalName": "$MULTIPLAYER_SCENARIO42_TITLE;",
                  "SignalType": "NavBeacon"
                }
              ],
              "horizons": true,
              "timestamp": "2025-09-08T17:49:26Z",
              "StarSystem": "Theta Persei",
              "SystemAddress": 1453586385251
            }
            "#;
            //TODO Does it make sense to save something here?
            return Ok(());
        }
        "SAAScanComplete" => {
            info!("Registered SAAScanComplete: {}", journal_id);
            return Ok(());
        }
        "Scan" => {
            let body_id = json["BodyID"].as_i32();
            let system_address = json["SystemAddress"].as_i64();
            let name = json["BodyName"].to_string();
            let radius = json["radius"].as_f32();
            let axial_tilt = json["AxialTilt"].as_f32();
            let mapped = json["WasMapped"].as_bool();
            let rotation_period = json["RotationPeriod"].as_f32();
            let surface_temperature = json["SurfaceTemperature"].as_f32();
            let distance = json["DistanceFromArrivalLS"].as_f32();

            if json.has_key("StarType") {
                //Star
                //language=json
                let _ = r#"
                {
                  "event": "Scan",
                  "Age_MY": 8334,
                  "BodyID": 0,
                  "Radius": 485754784.0,
                  "StarPos": [
                    81.90625,
                    -7.75,
                    64.25
                  ],
                  "odyssey": true,
                  "BodyName": "BPM 8422",
                  "ScanType": "AutoScan",
                  "StarType": "K",
                  "Subclass": 6,
                  "horizons": true,
                  "AxialTilt": 0,
                  "WasMapped": false,
                  "timestamp": "2025-09-08T17:53:37Z",
                  "Luminosity": "V",
                  "StarSystem": "BPM 8422",
                  "StellarMass": 0.554688,
                  "SystemAddress": 633742562018,
                  "WasDiscovered": true,
                  "RotationPeriod": 256538.422388,
                  "AbsoluteMagnitude": 7.996841,
                  "SurfaceTemperature": 4081.0,
                  "DistanceFromArrivalLS": 0
                }
                "#;
                let age_my = json["Age_MY"].as_i32();
                let star_type = json["StarType"].to_string();
                let subclass = json["Subclass"].as_i32();
                let luminosity = json["Luminosity"].to_string();
                let stellar_mass = json["StellarMass"].as_f32();
                let absolut_magnitude = json["AbsoluteMagnitude"].as_f32();
                match client.execute(
                    //language=postgresql
                    "INSERT INTO star (id, system_address, name, age_my, radius, star_type, subclass, axial_tilt, luminosity, stellar_mass,
                    rotation_period, absolut_magnitude, surface_temperature, distance, journal_id) VALUES
                    ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) ON CONFLICT ON CONSTRAINT star_pkey DO UPDATE SET
                    name=$3,age_my=$4,radius=$5,star_type=$6,subclass=$7,axial_tilt=$8,luminosity=$9,stellar_mass=$10,rotation_period=$11,absolut_magnitude=$12,surface_temperature=$13,distance=$14,journal_id=$15",
                    &[&body_id,&system_address,&name,&age_my,&radius,&star_type,&subclass,&axial_tilt,&luminosity,&stellar_mass,&rotation_period,&absolut_magnitude,&surface_temperature,&distance,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] inserting star: {}",journal_id, err);
                        return Err(EdcasError::from(err));
                    }
                }
            } else {
                //Body
                //language=json
                let _ = r#"
                {
                  "event": "Scan",
                  "BodyID": 11,
                  "MassEM": 0.079921,
                  "Radius": 2770988.75,
                  "Parents": [
                    {
                      "Star": 0
                    }
                  ],
                  "StarPos": [
                    1418.8125,
                    -263.625,
                    60976.65625
                  ],
                  "odyssey": true,
                  "BodyName": "Choi Dryoae NM-E c14-0 2",
                  "Landable": true,
                  "ScanType": "Detailed",
                  "horizons": true,
                  "AxialTilt": -3.094385,
                  "Materials": [
                    {
                      "Name": "iron",
                      "Percent": 22.63538
                    },
                    {
                      "Name": "nickel",
                      "Percent": 17.120453
                    },
                    {
                      "Name": "sulphur",
                      "Percent": 16.107216
                    },
                    {
                      "Name": "carbon",
                      "Percent": 13.544501
                    },
                    {
                      "Name": "chromium",
                      "Percent": 10.179882
                    },
                    {
                      "Name": "phosphorus",
                      "Percent": 8.671422
                    },
                    {
                      "Name": "germanium",
                      "Percent": 4.741801
                    },
                    {
                      "Name": "zirconium",
                      "Percent": 2.628434
                    },
                    {
                      "Name": "niobium",
                      "Percent": 1.547008
                    },
                    {
                      "Name": "tin",
                      "Percent": 1.471926
                    },
                    {
                      "Name": "yttrium",
                      "Percent": 1.351984
                    }
                  ],
                  "Periapsis": 201.245264,
                  "TidalLock": false,
                  "Volcanism": "",
                  "WasMapped": false,
                  "timestamp": "2025-09-08T17:53:32Z",
                  "Atmosphere": "thin carbon dioxide atmosphere",
                  "StarSystem": "Choi Dryoae NM-E c14-0",
                  "Composition": {
                    "Ice": 0,
                    "Rock": 0.670072,
                    "Metal": 0.329928
                  },
                  "MeanAnomaly": 221.621687,
                  "PlanetClass": "High metal content body",
                  "Eccentricity": 0.000179,
                  "AscendingNode": 155.684678,
                  "OrbitalPeriod": 43692640.066147,
                  "SemiMajorAxis": 175069940090.17944,
                  "SystemAddress": 86268265082,
                  "WasDiscovered": false,
                  "AtmosphereType": "CarbonDioxide",
                  "RotationPeriod": 104590.385641,
                  "SurfaceGravity": 4.148568,
                  "TerraformState": "",
                  "SurfacePressure": 3413.961182,
                  "OrbitalInclination": -0.013935,
                  "SurfaceTemperature": 212.139557,
                  "AtmosphereComposition": [
                    {
                      "Name": "CarbonDioxide",
                      "Percent": 92.788872
                    },
                    {
                      "Name": "SulphurDioxide",
                      "Percent": 7.211138
                    }
                  ],
                  "DistanceFromArrivalLS": 584.04864
                }
                "#;
                let mass_em = json["MassEM"].as_f32();
                let landable = json["Landable"].as_bool();
                let periapsis = json["Periapsis"].as_f32();
                let tidal_lock = json["TidalLock"].as_bool();
                let volcanism = value_table(
                    Tables::Volcanism,
                    json["Volcanism"].to_string(),
                    journal_id,
                    client,
                )?;
                let atmosphere = value_table(
                    Tables::Atmosphere,
                    json["Atmosphere"].to_string(),
                    journal_id,
                    client,
                )?;
                let mean_anomaly = json["MeanAnomaly"].as_f32();
                let planet_class = value_table(
                    Tables::PlanetClass,
                    json["PlanetClass"].to_string(),
                    journal_id,
                    client,
                )?;
                let eccentricity = json["Eccentricity"].as_f32();
                let ascending_node = json["AscendingNode"].as_f32();
                let orbital_period = json["OrbitalPeriod"].as_f32();
                let semi_major_axis = json["SemiMajorAxis"].as_f32();
                let atmosphere_type = value_table(
                    Tables::AtmosphereType,
                    json["AtmosphereType"].to_string(),
                    journal_id,
                    client,
                )?;
                let surface_gravity = json["SurfaceGravity"].as_f32();
                let terraform_state = value_table(
                    Tables::TerraformState,
                    json["TerraformState"].to_string(),
                    journal_id,
                    client,
                )?;
                let surface_pressure = json["SurfacePressure"].as_f32();
                let orbital_inclination = json["OrbitalInclination"].as_f32();

                match client.execute(
                    //language=postgresql
                    "INSERT INTO body
                        (id, system_address, name, mass_em, radius, landable, axial_tilt, periapsis, tidal_lock, volcanism, mapped, atmosphere,
                        mean_anomaly, planet_class, eccentricity, ascending_node, orbital_period, semi_major_axis, atmosphere_type, rotation_period,
                        surface_gravity, terraform_state, surface_pressure, orbital_inclination, surface_temperature, distance,journal_id)
                        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12, $13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27) ON CONFLICT ON CONSTRAINT body_pkey DO UPDATE SET
                        mass_em=$4,radius=$5,landable=$6,axial_tilt=$7,periapsis=$8,tidal_lock=$9,volcanism=$10,mapped=$11,atmosphere=$12,mean_anomaly=$13,
                        planet_class=$14,eccentricity=$15,ascending_node=$16,orbital_period=$17,semi_major_axis=$18,atmosphere_type=$19,rotation_period=$20,surface_gravity=$21,
                        terraform_state=$22,surface_pressure=$23,orbital_inclination=$24,surface_temperature=$25,distance=$26,journal_id=$27",
                    &[&body_id,&system_address,&name,&mass_em,&radius,&landable,&axial_tilt,&periapsis,&tidal_lock,&volcanism,&mapped,
                        &atmosphere,&mean_anomaly,&planet_class,&eccentricity,&ascending_node,&orbital_period,&semi_major_axis,&atmosphere_type,&rotation_period,
                        &surface_gravity,&terraform_state,&surface_pressure,&orbital_inclination,&surface_temperature,&distance,&journal_id]
                ){
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] inserting body: {}",journal_id, err);
                        return Err(EdcasError::from(err));
                    }
                }

                match client.execute(
                    //language=postgresql
                    "DELETE FROM atmosphere_composition WHERE body_id=$1 AND system_address=$2",
                    &[&body_id, &system_address],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] deleting atmosphere_composition: {}", journal_id, err);
                        return Err(EdcasError::from(err));
                    }
                }
                let atmosphere_composition_size = json["AtmosphereComposition"].len();
                for i in 0..atmosphere_composition_size {
                    let json = &json["AtmosphereComposition"][i];
                    let atmosphere_type = value_table(
                        Tables::AtmosphereType,
                        json["Name"].to_string(),
                        journal_id,
                        client,
                    )?;
                    let percent = json["Percent"].as_f32();
                    match client.execute(
                        //language=postgresql
                        "INSERT INTO atmosphere_composition (atmosphere_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
                        &[&atmosphere_type,&body_id,&system_address,&percent,&journal_id]
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] inserting atmosphere_composition: {}",journal_id, err);
                            return Err(EdcasError::from(err));
                        }
                    }
                }

                match client.execute(
                    //language=postgresql
                    "DELETE FROM planet_composition WHERE body_id=$1 AND system_address=$2",
                    &[&body_id, &system_address],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] deleting planet_composition: {}", journal_id, err);
                        return Err(EdcasError::from(err));
                    }
                }
                let compositions = &json["Composition"];
                for composition in compositions.entries() {
                    let composition_type = value_table(
                        Tables::PlanetCompositionType,
                        composition.0.to_string(),
                        journal_id,
                        client,
                    )?;
                    let percentage = composition.1.as_f32();
                    match client.execute(
                        //language=postgresql
                        "INSERT INTO planet_composition (composition_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
                        &[&composition_type,&body_id,&system_address,&percentage,&journal_id]
                    ){
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] inserting planet_composition: {}",journal_id, err);
                            return Err(EdcasError::from(err));
                        }
                    }
                }

                match client.execute(
                    //language=postgresql
                    "DELETE FROM planet_material WHERE body_id=$1 AND system_address=$2",
                    &[&body_id, &system_address],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] deleting planet_material: {}", journal_id, err);
                        return Err(EdcasError::from(err));
                    }
                }
                let material_size = json["Materials"].len();
                for i in 0..material_size {
                    let json = &json["Materials"][i];
                    let material_type = value_table(
                        Tables::MaterialType,
                        json["Name"].to_string(),
                        journal_id,
                        client,
                    )?;
                    let percent = json["Percent"].as_f32();
                    match client.execute(
                        //language=postgresql
                        "INSERT INTO planet_material (material_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
                        &[&material_type,&body_id,&system_address,&percent,&journal_id]
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] inserting planet_material: {}",journal_id, err);
                            return Err(EdcasError::from(err));
                        }
                    }
                }
            }

            if json.has_key("Parents") {
                let parent_size = json["Parents"].len();
                for i in 0..parent_size {
                    let parent = &json["Parents"][i];
                    for value in parent.entries() {
                        match client.execute(
                            //language=postgresql
                            "INSERT INTO parents (type, parent_id, body_id, system_address, journal_id) VALUES ($1,$2,$3,$4,$5) ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                            &[&value.0.to_string(),&value.1.as_i32(),&body_id,&system_address,&journal_id]
                        ){
                            Ok(_) => {}
                            Err(err) => {
                                error!("[{}] inserting parents: {}",journal_id, err);
                                return Err(EdcasError::from(err));
                            }
                        }
                    }
                }
            }
            return Ok(());
        }
        "ScanBaryCentre" => {
            //Planet scan with fss
            //language=json
            let _ = r#"
            {
              "event": "ScanBaryCentre",
              "BodyID": 2,
              "StarPos": [
                22.71875,
                281.4375,
                20.65625
              ],
              "odyssey": true,
              "horizons": true,
              "Periapsis": 112.432171,
              "timestamp": "2025-09-08T21:57:11Z",
              "StarSystem": "35 Comae Berenices",
              "MeanAnomaly": 152.771035,
              "Eccentricity": 0.199443,
              "AscendingNode": 46.292454,
              "OrbitalPeriod": 608515626192.0929,
              "SemiMajorAxis": 69470411539077.76,
              "SystemAddress": 908553589466,
              "OrbitalInclination": -37.675084
            }
            "#;
            //TODO probably won't need it?
        }

        //Maintenance
        "RefuelAll" => {
            info!("Registered RefuelAll: {}", journal_id);
            return Ok(());
        }
        "Resupply" => {
            info!("Registered Resupply: {}", journal_id);
            return Ok(());
        }
        "Repair" => {
            info!("Registered Repair: {}", journal_id);
            return Ok(());
        }
        "BuyDrones" => {
            info!("Registered BuyDrones: {}", journal_id);
            return Ok(());
        }
        "SellDrones" => {
            info!("Registered SellDrones: {}", journal_id);
            return Ok(());
        }
        "BuyAmmo" => {
            info!("Registered BuyAmmo: {}", journal_id);
            return Ok(());
        }
        "ReservoirReplenished" => {
            info!("Registered ReservoirReplenished: {}", journal_id);
            return Ok(());
        }
        "RepairAll" => {
            info!("Registered RepairAll: {}", journal_id);
            return Ok(());
        }
        "RebootRepair" => {
            info!("Registered RebootRepair: {}", journal_id);
            return Ok(());
        }
        "RestockVehicle" => {
            info!("Registered RestockVehicle: {}", journal_id);
            return Ok(());
        }

        //Docking
        "DockingRequested" => {
            info!("Registered DockingRequested: {}", journal_id);
            return Ok(());
        }
        "DockingGranted" => {
            //language=json
            let _ = r#"
            {
              "event": "DockingGranted",
              "odyssey": true,
              "MarketID": 4237424899,
              "horizons": true,
              "timestamp": "2025-09-08T22:14:24Z",
              "LandingPad": 32,
              "StationName": "Leibniz Point",
              "StationType": "Coriolis"
            }
            "#;
            //Probably nothing
            return Ok(());
        }
        "Docked" => {
            //language=json
            let _ = r#"
            {
              "Taxi": false,
              "event": "Docked",
              "StarPos": [
                11.1875,
                -37.375,
                -31.84375
              ],
              "odyssey": true,
              "MarketID": 3223415296,
              "horizons": true,
              "Multicrew": false,
              "timestamp": "2025-09-08T21:46:49Z",
              "StarSystem": "LHS 20",
              "LandingPads": {
                "Large": 6,
                "Small": 7,
                "Medium": 12
              },
              "StationName": "Ohm City",
              "StationType": "Coriolis",
              "SystemAddress": 33656303199641,
              "DistFromStarLS": 1289.578089,
              "StationEconomy": "$economy_HighTech;",
              "StationFaction": {
                "Name": "Movement for LHS 20 Liberals",
                "FactionState": "Boom"
              },
              "StationServices": [
                "dock",
                "autodock",
                "blackmarket",
                "commodities",
                "contacts",
                "exploration",
                "missions",
                "outfitting",
                "crewlounge",
                "rearm",
                "refuel",
                "repair",
                "shipyard",
                "tuning",
                "engineer",
                "missionsgenerated",
                "flightcontroller",
                "stationoperations",
                "powerplay",
                "searchrescue",
                "materialtrader",
                "techBroker",
                "stationMenu",
                "shop",
                "livery",
                "socialspace",
                "bartender",
                "vistagenomics",
                "pioneersupplies",
                "apexinterstellar",
                "frontlinesolutions",
                "registeringcolonisation"
              ],
              "StationEconomies": [
                {
                  "Name": "$economy_HighTech;",
                  "Proportion": 0.8
                },
                {
                  "Name": "$economy_Refinery;",
                  "Proportion": 0.2
                }
              ],
              "StationAllegiance": "Federation",
              "StationGovernment": "$government_Democracy;"
            }
            "#;
            let market_id = json["MarketID"]
                .as_i64()
                .ok_or(format!("[{}] No MarketID in json", journal_id))?;
            let system_allegiance = value_table(
                Tables::Allegiance,
                json["SystemAllegiance"].to_string(),
                journal_id,
                client,
            )?;
            let controlling_power = if json.has_key("ControllingPower") {
                Some(value_table(
                    Tables::Power,
                    json["ControllingPower"].to_string(),
                    journal_id,
                    client,
                )?)
            } else {
                None
            };
            let system_address = star_system::insert_star_system(
                json["SystemAddress"]
                    .as_i64()
                    .ok_or(format!("[{}]No SystemAddress in json", journal_id))?,
                json["StarSystem"].to_string(),
                (
                    json["StarPos"][0]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos x in json", journal_id))?,
                    json["StarPos"][1]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos y in json", journal_id))?,
                    json["StarPos"][2]
                        .as_f32()
                        .ok_or(format!("[{}] No StarPos z in json", journal_id))?,
                ),
                system_allegiance,
                value_table(
                    Tables::EconomyType,
                    json["SystemEconomy"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::EconomyType,
                    json["SystemSecondEconomy"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::Government,
                    json["SystemGovernment"].to_string(),
                    journal_id,
                    client,
                )?,
                value_table(
                    Tables::Security,
                    json["SystemSecurity"].to_string(),
                    journal_id,
                    client,
                )?,
                json["Population"].as_i64().unwrap_or_default(),
                controlling_power,
                journal_id,
                client,
            )?;
            faction::insert_factions(&json, client, &system_address, journal_id)?;
            if json.has_key("Conflicts") {
                faction::insert_conflict(&json, client, &system_address, journal_id)?;
            }
            let faction_name = value_table(
                Tables::FactionName,
                json["StationFaction"]["Name"].to_string(),
                journal_id,
                client,
            )?;
            let government = value_table(
                Tables::Government,
                json["StationGovernment"].to_string(),
                journal_id,
                client,
            )?;
            let economy = value_table(
                Tables::EconomyType,
                json["StationEconomy"].to_string(),
                journal_id,
                client,
            )?;
            let station_name = json["StationName"].to_string();
            let station_type = value_table(
                Tables::StationType,
                json["StationType"].to_string(),
                journal_id,
                client,
            )?;
            insert_station_factions(
                client,
                &json,
                faction_name,
                government,
                system_allegiance,
                &system_address,
                journal_id,
            )?;

            let market_available = match client.query_one(
                // language=postgresql
                "SELECT 1 FROM stations WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(row) => {
                    if row.is_empty() {
                        false
                    } else {
                        true
                    }
                }
                Err(err) => {
                    if err.to_string() != "query returned an unexpected number of rows" {
                        error!(
                            "[{}]insert_station: Unable to get station: {}",
                            journal_id, err
                        );
                        return Err(EdcasError::from(err));
                    }
                    false
                }
            };
            if market_available {
                //Update
                match client.execute(
                    // language=postgresql
                    "UPDATE stations
                            SET
                                system_address=$1,
                                name=$2,
                                type=$3,
                                faction_name=$4,
                                government=$5,
                                economy=$6,
                                journal_id=$7
                            WHERE market_id=$8
                                ",
                    &[
                        &system_address,
                        &station_name,
                        &station_type,
                        &faction_name,
                        &government,
                        &economy,
                        &journal_id,
                        &market_id,
                    ],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "[{}]insert_station: Unable to update station: {}",
                            journal_id, err
                        );
                    }
                }
            } else {
                //Insert
                match client.execute(
                    // language=postgresql
                    "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
                            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                    &[&market_id,&system_address,&station_name,&station_type,&faction_name,&government,&economy,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}]insert_station: Unable to insert station: {}",journal_id,err);
                    }
                }
            }
            match client.execute(
                // language=postgresql
                "DELETE FROM station_services WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "[{}]Insert station services: couldn't delete old station service: {}",
                        journal_id, err
                    );
                }
            }
            let station_services_size = json["StationServices"].len();
            for i in 0..station_services_size {
                let id = value_table(
                    Tables::StationServicesTypes,
                    json["StationServices"][i].to_string(),
                    journal_id,
                    client,
                )?;
                match client.execute(
                    // language=postgresql
                    "INSERT INTO station_services (id, market_id,journal_id) VALUES ($1, $2,$3)",
                    &[&id, &market_id, &journal_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "[{}]Insert station services: couldn't insert station service: {}",
                            journal_id, err
                        );
                    }
                }
            }
            let economy_size = json["StationEconomies"].len();
            if economy_size > 0 {
                //Delete all existing facion, since an update comes in
                match client.execute(
                    // language=postgresql
                    "DELETE FROM station_economies WHERE market_id = $1",
                    &[&market_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] station_economies: Couldn't delete old economy state from station {}: {}",journal_id,market_id,err);
                        return Err(EdcasError::from(err));
                    }
                }
            }
            for i in 0..economy_size {
                let json = &json["StationEconomies"][i];
                let economy = value_table(
                    Tables::EconomyType,
                    json["Name"].to_string(),
                    journal_id,
                    client,
                )?;
                let proportion = json["Proportion"].as_f32().unwrap_or_default();
                match client.execute(
                    // language=postgresql
                    "INSERT INTO station_economies (id, market_id, proportion,journal_id) VALUES ($1, $2, $3,$4)",
                    &[&economy,&market_id,&proportion,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] station_economies: Couldn't insert station economy: {}",journal_id,err);
                    }
                }
            }

            if json.has_key("LandingPads") {
                match client.execute(
                    //language=postgresql
                    "DELETE FROM station_landing_pads WHERE market_id=$1",
                    &[&market_id],
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] landing_pads: Couldn't delete old landing_pads from station {}: {}",journal_id,market_id,err);
                    }
                }
                let json = &json["LandingPads"];
                for entry in json.entries() {
                    let landing_pat_types = value_table(
                        Tables::LandingPadsTypes,
                        entry.0.to_string(),
                        journal_id,
                        client,
                    )?;
                    let count = entry.1.as_i32();
                    match client.execute(
                        // language=postgresql
                        "INSERT INTO station_landing_pads (market_id, landing_pads_type, count, journal_id) VALUES ($1,$2,$3,$4)",
                        &[&market_id,&landing_pat_types,&count,&journal_id]
                    ){
                        Ok(_) => {}
                        Err(err) => {
                            error!("[{}] landing_pads: Couldn't insert landing_pad from station {}: {}",journal_id,market_id,err);
                        }
                    }
                }
            }
            return Ok(());
        }
        "Undocked" => {
            //{ "timestamp":"2023-09-09T18:29:17Z", "event":"Undocked", "StationName":"Q2K-BHB", "StationType":"FleetCarrier", "MarketID":3704402432, "Taxi":false, "Multicrew":false }
            info!("Registered Undocked: {}", journal_id);
            return Ok(());
        }

        //Engineer
        "EngineerProgress" => {
            info!("Registered EngineerProgress: {}", journal_id);
            return Ok(());
        }
        "EngineerCraft" => {
            //{ "timestamp":"2023-12-05T20:54:13Z", "event":"EngineerCraft", "Slot":"PowerDistributor",
            // "Module":"int_powerdistributor_size7_class5",
            // "Ingredients":[
            // { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":1 },
            // { "Name":"industrialfirmware", "Name_Localised":"Gecrackte Industrie-Firmware", "Count":1 },
            // { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":1 } ],
            // "Engineer":"The Dweller", "EngineerID":300180, "BlueprintID":128673738, "BlueprintName":"PowerDistributor_HighFrequency",
            // "Level":4, "Quality":0.267800, "ExperimentalEffect":"special_powerdistributor_fast",
            // "ExperimentalEffect_Localised":"Superleiter",
            // "Modifiers":[
            // { "Label":"WeaponsCapacity", "Value":56.217598, "OriginalValue":61.000000, "LessIsGood":0 }, { "Label":"WeaponsRecharge", "Value":8.209770, "OriginalValue":6.100000, "LessIsGood":0 }, { "Label":"EnginesCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"EnginesRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 }, { "Label":"SystemsCapacity", "Value":37.785599, "OriginalValue":41.000000, "LessIsGood":0 }, { "Label":"SystemsRecharge", "Value":5.383456, "OriginalValue":4.000000, "LessIsGood":0 } ] }
            info!("Registered EngineerCraft: {}", journal_id);
            return Ok(());
        }
        "EngineerContribution" => {
            info!("Registered EngineerContribution: {}", journal_id);
            return Ok(());
        }

        //Ship management
        "Shipyard" => {
            info!("Registered Shipyard: {}", journal_id);
            return Ok(());
        }
        "StoredShips" => {
            info!("Registered StoredShips: {}", journal_id);
            return Ok(());
        }
        "ShipyardSwap" => {
            info!("Registered ShipyardSwap: {}", journal_id);
            return Ok(());
        }
        "ShipLocker" => {
            info!("Registered ShipLocker: {}", journal_id);
            return Ok(());
        }
        "ModuleBuy" => {
            info!("Registered ModuleBuy: {}", journal_id);
            return Ok(());
        }
        "Outfitting" => {
            info!("Registered Outfitting: {}", journal_id);
            return Ok(());
        }
        "ModuleInfo" => {
            info!("Registered ModuleInfo: {}", journal_id);
            return Ok(());
        }
        "StoredModules" => {
            info!("Registered StoredModules: {}", journal_id);
            return Ok(());
        }
        "DockingCancelled" => {
            info!("Registered DockingCancelled: {}", journal_id);
            return Ok(());
        }
        "ShipyardBuy" => {
            info!("Registered ShipyardBuy: {}", journal_id);
            return Ok(());
        }
        "ShipyardNew" => {
            info!("Registered ShipyardNew: {}", journal_id);
            return Ok(());
        }
        "ShipyardTransfer" => {}
        "ModuleStore" => {}
        "ModuleSell" => {}
        "ModuleSellRemote" => {}
        "ModuleSwap" => {}

        //On foot
        "Backpack" => {}
        "BackpackChange" => {}
        "CollectItems" => {}
        "UpgradeSuit" => {}
        "Loadout" => {}
        "LoadoutEquipModule" => {}
        "SuitLoadout" => {}
        "UseConsumable" => {}
        "ScanOrganic" => {}
        "BuyWeapon" => {}

        //Market
        "MarketBuy" => {}
        "Market" => {}
        "MarketSell" => {}

        //SRV
        "LaunchSRV" => {}
        "DockSRV" => {}

        //Ship fight
        "ShipTargeted" => {}
        "UnderAttack" => {}
        "ShieldState" => {}
        "HullDamage" => {}

        //Cargo, Materials & Mining & Drones
        //{ "timestamp":"2022-09-07T20:08:23Z", "event":"Materials",
        // "Raw":[ { "Name":"sulphur", "Name_Localised":"Schwefel", "Count":300 }, { "Name":"manganese", "Name_Localised":"Mangan", "Count":236 }, { "Name":"vanadium", "Count":95 }, { "Name":"nickel", "Count":300 }, { "Name":"phosphorus", "Name_Localised":"Phosphor", "Count":296 }, { "Name":"iron", "Name_Localised":"Eisen", "Count":300 }, { "Name":"germanium", "Count":239 }, { "Name":"chromium", "Name_Localised":"Chrom", "Count":213 }, { "Name":"carbon", "Name_Localised":"Kohlenstoff", "Count":257 }, { "Name":"molybdenum", "Name_Localised":"Molibdän", "Count":153 }, { "Name":"cadmium", "Name_Localised":"Kadmium", "Count":13 }, { "Name":"selenium", "Name_Localised":"Selen", "Count":14 }, { "Name":"mercury", "Name_Localised":"Quecksilber", "Count":19 }, { "Name":"yttrium", "Count":22 }, { "Name":"zinc", "Name_Localised":"Zink", "Count":250 }, { "Name":"ruthenium", "Count":24 }, { "Name":"arsenic", "Name_Localised":"Arsen", "Count":24 }, { "Name":"tungsten", "Name_Localised":"Wolfram", "Count":75 }, { "Name":"tellurium", "Name_Localised":"Tellur", "Count":12 }, { "Name":"tin", "Name_Localised":"Zinn", "Count":131 }, { "Name":"antimony", "Name_Localised":"Antimon", "Count":45 }, { "Name":"niobium", "Name_Localised":"Niob", "Count":44 }, { "Name":"zirconium", "Count":48 }, { "Name":"technetium", "Count":39 }, { "Name":"lead", "Name_Localised":"Blei", "Count":90 }, { "Name":"boron", "Name_Localised":"Bor", "Count":14 }, { "Name":"polonium", "Count":8 } ],
        // "Manufactured":[ { "Name":"hybridcapacitors", "Name_Localised":"Hybridkondensatoren", "Count":197 }, { "Name":"heatdispersionplate", "Name_Localised":"Wärmeverteilungsplatte", "Count":67 }, { "Name":"gridresistors", "Name_Localised":"Gitterwiderstände", "Count":242 }, { "Name":"mechanicalequipment", "Name_Localised":"Mechanisches Equipment", "Count":220 }, { "Name":"fedcorecomposites", "Name_Localised":"Core Dynamics Kompositwerkstoffe", "Count":100 }, { "Name":"protoheatradiators", "Name_Localised":"Proto-Wärmestrahler", "Count":6 }, { "Name":"salvagedalloys", "Name_Localised":"Geborgene Legierungen", "Count":300 }, { "Name":"highdensitycomposites", "Name_Localised":"Komposite hoher Dichte", "Count":200 }, { "Name":"mechanicalscrap", "Name_Localised":"Mechanischer Schrott", "Count":64 }, { "Name":"chemicalprocessors", "Name_Localised":"Chemische Prozessoren", "Count":250 }, { "Name":"focuscrystals", "Name_Localised":"Laserkristalle", "Count":200 }, { "Name":"imperialshielding", "Name_Localised":"Imperiale Schilde", "Count":53 }, { "Name":"precipitatedalloys", "Name_Localised":"Gehärtete Legierungen", "Count":200 }, { "Name":"galvanisingalloys", "Name_Localised":"Galvanisierende Legierungen", "Count":250 }, { "Name":"shieldingsensors", "Name_Localised":"Schildsensoren", "Count":200 }, { "Name":"chemicaldistillery", "Name_Localised":"Chemiedestillerie", "Count":200 }, { "Name":"heatconductionwiring", "Name_Localised":"Wärmeleitungsverdrahtung", "Count":128 }, { "Name":"phasealloys", "Name_Localised":"Phasenlegierungen", "Count":195 }, { "Name":"wornshieldemitters", "Name_Localised":"Gebrauchte Schildemitter", "Count":300 }, { "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":250 }, { "Name":"mechanicalcomponents", "Name_Localised":"Mechanische Komponenten", "Count":11 }, { "Name":"compoundshielding", "Name_Localised":"Verbundschilde", "Count":150 }, { "Name":"protolightalloys", "Name_Localised":"Leichte Legierungen (Proto)", "Count":145 }, { "Name":"refinedfocuscrystals", "Name_Localised":"Raffinierte Laserkristalle", "Count":150 }, { "Name":"heatexchangers", "Name_Localised":"Wärmeaustauscher", "Count":6 }, { "Name":"conductiveceramics", "Name_Localised":"Elektrokeramiken", "Count":44 }, { "Name":"uncutfocuscrystals", "Name_Localised":"Fehlerhafte Fokuskristalle", "Count":250 }, { "Name":"temperedalloys", "Name_Localised":"Vergütete Legierungen", "Count":92 }, { "Name":"basicconductors", "Name_Localised":"Einfache Leiter", "Count":140 }, { "Name":"crystalshards", "Name_Localised":"Kristallscherben", "Count":288 }, { "Name":"unknownenergycell", "Name_Localised":"Thargoiden-Energiezelle", "Count":171 }, { "Name":"unknowntechnologycomponents", "Name_Localised":"Technologiekomponenten der Thargoiden", "Count":150 }, { "Name":"unknownenergysource", "Name_Localised":"Sensorenfragment", "Count":100 }, { "Name":"unknowncarapace", "Name_Localised":"Thargoiden-Krustenschale", "Count":220 }, { "Name":"unknownorganiccircuitry", "Name_Localised":"Organischer Schaltkreis der Thargoiden", "Count":100 }, { "Name":"chemicalmanipulators", "Name_Localised":"Chemische Manipulatoren", "Count":72 }, { "Name":"exquisitefocuscrystals", "Name_Localised":"Erlesene Laserkristalle", "Count":89 }, { "Name":"configurablecomponents", "Name_Localised":"Konfigurierbare Komponenten", "Count":36 }, { "Name":"heatvanes", "Name_Localised":"Wärmeleitbleche", "Count":1 }, { "Name":"biotechconductors", "Name_Localised":"Biotech-Leiter", "Count":57 }, { "Name":"conductivepolymers", "Name_Localised":"Leitfähige Polymere", "Count":5 }, { "Name":"thermicalloys", "Name_Localised":"Thermische Legierungen", "Count":150 }, { "Name":"conductivecomponents", "Name_Localised":"Leitfähige Komponenten", "Count":169 }, { "Name":"fedproprietarycomposites", "Name_Localised":"Kompositwerkstoffe", "Count":150 }, { "Name":"electrochemicalarrays", "Name_Localised":"Elektrochemische Detektoren", "Count":133 }, { "Name":"compactcomposites", "Name_Localised":"Kompaktkomposite", "Count":101 }, { "Name":"filamentcomposites", "Name_Localised":"Filament-Komposite", "Count":250 }, { "Name":"chemicalstorageunits", "Name_Localised":"Lagerungseinheiten für Chemiestoffe", "Count":57 }, { "Name":"protoradiolicalloys", "Name_Localised":"Radiologische Legierungen (Proto)", "Count":39 }, { "Name":"guardian_powercell", "Name_Localised":"Guardian-Energiezelle", "Count":300 }, { "Name":"guardian_powerconduit", "Name_Localised":"Guardian-Energieleiter", "Count":250 }, { "Name":"guardian_techcomponent", "Name_Localised":"Guardian-Technologiekomponenten", "Count":160 }, { "Name":"guardian_sentinel_weaponparts", "Name_Localised":"Guardian-Wache-Waffenteile", "Count":200 }, { "Name":"pharmaceuticalisolators", "Name_Localised":"Pharmazeutische Isolatoren", "Count":27 }, { "Name":"militarygradealloys", "Name_Localised":"Militärqualitätslegierungen", "Count":63 }, { "Name":"guardian_sentinel_wreckagecomponents", "Name_Localised":"Guardian-Wrackteilkomponenten", "Count":300 }, { "Name":"heatresistantceramics", "Name_Localised":"Hitzefeste Keramik", "Count":87 }, { "Name":"polymercapacitors", "Name_Localised":"Polymerkondensatoren", "Count":91 }, { "Name":"tg_biomechanicalconduits", "Name_Localised":"Biomechanische Leiter", "Count":105 }, { "Name":"tg_wreckagecomponents", "Name_Localised":"Wrackteilkomponenten", "Count":144 }, { "Name":"tg_weaponparts", "Name_Localised":"Waffenteile", "Count":135 }, { "Name":"tg_propulsionelement", "Name_Localised":"Schubantriebelemente", "Count":100 }, { "Name":"militarysupercapacitors", "Name_Localised":"Militärische Superkondensatoren", "Count":1 }, { "Name":"improvisedcomponents", "Name_Localised":"Behelfskomponenten", "Count":4 } ],
        // "Encoded":[ { "Name":"shielddensityreports", "Name_Localised":"Untypische Schildscans ", "Count":200 }, { "Name":"shieldcyclerecordings", "Name_Localised":"Gestörte Schildzyklus-Aufzeichnungen", "Count":234 }, { "Name":"encryptedfiles", "Name_Localised":"Ungewöhnliche verschlüsselte Files", "Count":92 }, { "Name":"bulkscandata", "Name_Localised":"Anormale Massen-Scan-Daten", "Count":192 }, { "Name":"decodedemissiondata", "Name_Localised":"Entschlüsselte Emissionsdaten", "Count":112 }, { "Name":"encryptioncodes", "Name_Localised":"Getaggte Verschlüsselungscodes", "Count":33 }, { "Name":"shieldsoakanalysis", "Name_Localised":"Inkonsistente Schildleistungsanalysen", "Count":250 }, { "Name":"scanarchives", "Name_Localised":"Unidentifizierte Scan-Archive", "Count":112 }, { "Name":"disruptedwakeechoes", "Name_Localised":"Atypische FSA-Stör-Aufzeichnungen", "Count":228 }, { "Name":"archivedemissiondata", "Name_Localised":"Irreguläre Emissionsdaten", "Count":65 }, { "Name":"legacyfirmware", "Name_Localised":"Spezial-Legacy-Firmware", "Count":78 }, { "Name":"scrambledemissiondata", "Name_Localised":"Außergewöhnliche verschlüsselte Emissionsdaten", "Count":84 }, { "Name":"encodedscandata", "Name_Localised":"Divergente Scandaten", "Count":30 }, { "Name":"fsdtelemetry", "Name_Localised":"Anormale FSA-Telemetrie", "Count":123 }, { "Name":"wakesolutions", "Name_Localised":"Seltsame FSA-Zielorte", "Count":93 }, { "Name":"emissiondata", "Name_Localised":"Unerwartete Emissionsdaten", "Count":142 }, { "Name":"shieldpatternanalysis", "Name_Localised":"Abweichende Schildeinsatz-Analysen", "Count":78 }, { "Name":"scandatabanks", "Name_Localised":"Scan-Datenbanken unter Verschluss", "Count":68 }, { "Name":"consumerfirmware", "Name_Localised":"Modifizierte Consumer-Firmware", "Count":48 }, { "Name":"symmetrickeys", "Name_Localised":"Offene symmetrische Schlüssel", "Count":24 }, { "Name":"shieldfrequencydata", "Name_Localised":"Verdächtige Schildfrequenz-Daten", "Count":50 }, { "Name":"compactemissionsdata", "Name_Localised":"Anormale kompakte Emissionsdaten", "Count":18 }, { "Name":"adaptiveencryptors", "Name_Localised":"Adaptive Verschlüsselungserfassung", "Count":64 }, { "Name":"encryptionarchives", "Name_Localised":"Atypische Verschlüsselungsarchive", "Count":63 }, { "Name":"dataminedwake", "Name_Localised":"FSA-Daten-Cache-Ausnahmen", "Count":19 }, { "Name":"securityfirmware", "Name_Localised":"Sicherheits-Firmware-Patch", "Count":29 }, { "Name":"embeddedfirmware", "Name_Localised":"Modifizierte integrierte Firmware", "Count":58 }, { "Name":"tg_residuedata", "Name_Localised":"Thargoiden-Rückstandsdaten", "Count":55 }, { "Name":"tg_compositiondata", "Name_Localised":"Materialzusammensetzungsdaten der Thargoiden", "Count":49 }, { "Name":"tg_structuraldata", "Name_Localised":"Thargoiden-Strukturdaten", "Count":49 }, { "Name":"unknownshipsignature", "Name_Localised":"Thargoiden-Schiffssignatur", "Count":37 }, { "Name":"unknownwakedata", "Name_Localised":"Thargoiden-Sogwolkendaten", "Count":55 }, { "Name":"ancienthistoricaldata", "Name_Localised":"Gamma-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancienttechnologicaldata", "Name_Localised":"Epsilon-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientbiologicaldata", "Name_Localised":"Alpha-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientlanguagedata", "Name_Localised":"Delta-Muster-Obeliskendaten", "Count":150 }, { "Name":"ancientculturaldata", "Name_Localised":"Beta-Muster-Obeliskendaten", "Count":150 }, { "Name":"classifiedscandata", "Name_Localised":"Geheimes Scan-Fragment", "Count":18 }, { "Name":"hyperspacetrajectories", "Name_Localised":"Exzentrische Hyperraum-Routen", "Count":104 }, { "Name":"guardian_weaponblueprint", "Name_Localised":"Guardian-Waffenbauplanfragment", "Count":4 }, { "Name":"guardian_moduleblueprint", "Name_Localised":"Guardian-Modulbauplanfragment", "Count":7 }, { "Name":"guardian_vesselblueprint", "Name_Localised":"Guardian-Schiffsbauplanfragment", "Count":8 }, { "Name":"tg_shipflightdata", "Name_Localised":"Schiffsflugdaten", "Count":18 }, { "Name":"tg_shipsystemsdata", "Name_Localised":"Schiffssysteme-Daten", "Count":45 } ] }
        "Materials" => {}
        "Cargo" => {}
        "MaterialCollected" => {
            //{ "timestamp":"2023-12-05T19:44:43Z", "event":"MaterialCollected", "Category":"Manufactured", "Name":"shieldemitters", "Name_Localised":"Schildemitter", "Count":3 }            let material_category = json["Category"].to_string();
        }
        "Synthesis" => {}
        "EjectCargo" => {}
        "DropItems" => {}
        "LaunchDrone" => {}
        "MiningRefined" => {}
        "ProspectedAsteroid" => {
            //{ "timestamp":"2023-06-05T12:05:12Z", "event":"ProspectedAsteroid", "Materials":[ { "Name":"rutile", "Name_Localised":"Rutil", "Proportion":35.986309 }, { "Name":"Bauxite", "Name_Localised":"Bauxit", "Proportion":13.713245 } ], "Content":"$AsteroidMaterialContent_Low;", "Content_Localised":"Materialgehalt: Niedrig", "Remaining":100.000000 }
        }
        "CargoTransfer" => {}
        "CollectCargo" => {}

        //Mission and Redeeming
        "Missions" => {}
        "MissionAccepted" => {}
        "MissionRedirected" => {}
        "MissionCompleted" => {}
        "RedeemVoucher" => {}
        "Bounty" => {}
        "NpcCrewPaidWage" => {}
        "PayFines" => {}
        "MissionAbandoned" => {}
        "MissionFailed" => {}
        "PayBounties" => {}
        "SellOrganicData" => {}

        //Carrier
        "CarrierStats" => {}
        "CarrierJumpRequest" => {}
        "CarrierTradeOrder" => {}
        "CarrierFinance" => {}
        "CarrierJumpCancelled" => {}
        "CarrierDepositFuel" => {}
        "CarrierDockingPermission" => {}
        "CarrierCrewServices" => {}
        "CarrierModulePack" => {}
        "CarrierBankTransfer" => {}

        //Dropship
        "BookDropship" => {}
        "DropshipDeploy" => {}

        //Wing
        "WingInvite" => {}
        "WingJoin" => {}
        "WingAdd" => {}
        "WingLeave" => {}

        //Crew
        "CrewMemberQuits" => {}
        "CrewMemberRoleChange" => {}
        "CrewMemberJoins" => {}
        "EndCrewSession" => {}

        "SellMicroResources" => {}
        "TradeMicroResources" => {}
        "FuelScoop" => {}
        "ReceiveText" => {}
        "Friends" => {}
        "Scanned" => {}
        "LoadGame" => {}
        "SquadronStartup" => {}
        "Music" => {}
        "CodexEntry" => {}
        "Rank" => {}
        "Progress" => {}
        "Reputation" => {}
        "Statistics" => {}
        "Commander" => {}
        "PowerplaySalary" => {}
        "Powerplay" => {}
        "CommitCrime" => {}
        "DockingDenied" => {}
        "HeatWarning" => {}
        "FactionKillBond" => {}
        "MultiSellExplorationData" => {}
        "SwitchSuitLoadout" => {}
        "MaterialTrade" => {
            //{ "timestamp":"2023-12-05T19:23:23Z", "event":"MaterialTrade", "MarketID":3223208960, "TraderType":"manufactured",
            // "Paid":{ "Material":"fedcorecomposites", "Material_Localised":"Core Dynamics Kompositwerkstoffe", "Category":"Manufactured", "Quantity":6 },
            // "Received":{ "Material":"protoradiolicalloys", "Material_Localised":"Radiologische Legierungen (Proto)", "Category":"Manufactured", "Quantity":1 } }
        }
        "CommunityGoal" => {}
        "ModuleRetrieve" => {}
        "FetchRemoteModule" => {}
        "SendText" => {}
        "SearchAndRescue" => {}
        "HeatDamage" => {}
        "CommunityGoalReward" => {}
        "NavBeaconScan" => {}
        "USSDrop" => {}
        "Interdicted" => {}
        "Promotion" => {}
        "RepairDrone" => {}
        "DataScanned" => {}
        "DatalinkScan" => {}
        "DatalinkVoucher" => {}
        "CockpitBreached" => {}
        "SystemsShutdown" => {}
        "Screenshot" => {}
        "UpgradeWeapon" => {}
        "PowerplayFastTrack" => {}
        "PowerplayCollect" => {}
        "PowerplayDeliver" => {}
        "BookTaxi" => {}
        "SharedBookmarkToSquadron" => {}
        "MaterialDiscovered" => {}
        "SetUserShipName" => {}
        "FCMaterials" => {}
        "CommunityGoalJoin" => {}
        "SupercruiseDestinationDrop" => {}
        "JetConeBoost" => {}
        "AsteroidCracked" => {}
        "EscapeInterdiction" => {}
        "TechnologyBroker" => {}
        "NavBeaconDetail" => {}

        //Jesus
        "Died" => {}
        "Resurrect" => {}
        "SelfDestruct" => {}

        //Redeem
        "ShipyardRedeem" => {}
        "ShipRedeemed" => {}

        //Misc
        "commodities" => {
            let commodities_size = json["commodities"].len();
            let market_id = match json["marketId"].as_i64() {
                None => {
                    return Err(EdcasError {
                        0: "No market_id for commodities".to_string(),
                    })
                }
                Some(value) => value,
            };
            match client.execute(
                // language=postgresql
                "DELETE FROM commodity_listening WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "[{}] Couldn't delete old commodity_listening: {}",
                        journal_id, err
                    );
                }
            }
            for i in 0..commodities_size {
                let json = &json["commodities"][i];

                let name = value_table(
                    Tables::CommodityName,
                    json["name"].to_string(),
                    journal_id,
                    client,
                )?;

                let buy_price = json["buyPrice"].as_i32();
                let demand = json["demand"].as_i32();
                let demand_bracket = json["demandBracket"].as_i32();
                let mean_price = json["meanPrice"].as_i32();
                let sell_price = json["sellPrice"].as_i32();
                let stock = json["stock"].as_i32();
                let stock_bracket = json["stockBracket"].as_i32();
                match client.execute(
                    // language=postgresql
                    "INSERT INTO commodity_listening (commodity_name, market_id, buy_price, demand, demand_bracket, mean_price, sell_price, stock, stock_bracket, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
                    &[&name,&market_id,&buy_price,&demand,&demand_bracket,&mean_price,&sell_price,&stock,&stock_bracket,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] Couldn't insert commodity_listening: {}",journal_id,err);
                    }
                }
            }
            return Ok(());
        }
        "ships" => {
            //Ships
            //{"horizons":true,"marketId":4253567491,"odyssey":true,"ships":["anaconda","asp","asp_scout","belugaliner","cobramkiii","cobramkv","corsair","diamondback","diamondbackxl","dolphin","eagle","ferdelance","hauler","independant_trader","krait_light","krait_mkii","mamba","mandalay","panthermkii","python","python_nx","sidewinder","type6","type7","type8","type9","type9_military","typex_2","typex_3","viper","viper_mkiv","vulture"],"stationName":"Soundand Vision","systemName":"Delta-2 Chamaelontis","timestamp":"2025-09-08T14:35:56Z"}
            let market_id = match json["marketId"].as_i64() {
                None => {
                    return Err(EdcasError {
                        0: "ships: No market id".to_string(),
                    });
                }
                Some(value) => value,
            };
            match client.execute(
                //language=postgresql
                "DELETE FROM ship_listening WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "[{}] Couldn't delete old ship listenings: {}",
                        journal_id, err
                    );
                }
            }
            let ship_size = json["ships"].len();
            for i in 0..ship_size {
                let ship_name = value_table(
                    Tables::ShipName,
                    json["ships"][i].to_string(),
                    journal_id,
                    client,
                )?;
                match client.execute(
                    //language=postgresql
                    "INSERT INTO ship_listening (ship_name, market_id, journal_id) VALUES ($1,$2,$3)",
                    &[&ship_name,&market_id,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] Couldn't insert ship listening: {}",journal_id,err);
                    }
                }
            }
            return Ok(());
        }
        "modules" => {
            //{"horizons":true,"marketId":4253567491,"modules":["Hpt_advancedtorppylon_fixed_large","Hpt_advancedtorppylon_fixed_medium","Hpt_advancedtorppylon_fixed_small","Hpt_basicmissilerack_fixed_large","Hpt_basicmissilerack_fixed_medium","Hpt_basicmissilerack_fixed_small","Hpt_beamlaser_fixed_huge","Hpt_beamlaser_fixed_large","Hpt_beamlaser_fixed_medium","Hpt_beamlaser_fixed_small","Hpt_beamlaser_gimbal_huge","Hpt_beamlaser_gimbal_large","Hpt_beamlaser_gimbal_medium","Hpt_beamlaser_gimbal_small","Hpt_beamlaser_turret_small","Hpt_cannon_fixed_huge","Hpt_cannon_fixed_large","Hpt_cannon_fixed_medium","Hpt_cannon_gimbal_huge","Hpt_cannon_gimbal_large","Hpt_cannon_gimbal_medium","Hpt_cannon_gimbal_small","Hpt_cannon_turret_large","Hpt_cannon_turret_medium","Hpt_cannon_turret_small","Hpt_cargoscanner_size0_class1","Hpt_cargoscanner_size0_class2","Hpt_cargoscanner_size0_class3","Hpt_cargoscanner_size0_class4","Hpt_cargoscanner_size0_class5","Hpt_chafflauncher_tiny","Hpt_cloudscanner_size0_class1","Hpt_cloudscanner_size0_class2","Hpt_cloudscanner_size0_class3","Hpt_cloudscanner_size0_class4","Hpt_cloudscanner_size0_class5","Hpt_crimescanner_size0_class1","Hpt_crimescanner_size0_class2","Hpt_crimescanner_size0_class3","Hpt_crimescanner_size0_class4","Hpt_crimescanner_size0_class5","Hpt_dumbfiremissilerack_fixed_large","Hpt_dumbfiremissilerack_fixed_medium","Hpt_dumbfiremissilerack_fixed_small","Hpt_electroniccountermeasure_tiny","Hpt_heatsinklauncher_turret_tiny","Hpt_minelauncher_fixed_small","Hpt_minelauncher_fixed_small_impulse","Hpt_mining_abrblstr_fixed_small","Hpt_mining_seismchrgwarhd_fixed_medium","Hpt_mining_subsurfdispmisle_fixed_medium","Hpt_mining_subsurfdispmisle_fixed_small","Hpt_mining_subsurfdispmisle_turret_medium","Hpt_mininglaser_turret_small","Hpt_mrascanner_size0_class1","Hpt_mrascanner_size0_class2","Hpt_mrascanner_size0_class5","Hpt_multicannon_fixed_huge","Hpt_multicannon_fixed_large","Hpt_multicannon_fixed_medium","Hpt_multicannon_fixed_small","Hpt_multicannon_gimbal_huge","Hpt_multicannon_gimbal_large","Hpt_multicannon_gimbal_medium","Hpt_multicannon_gimbal_small","Hpt_multicannon_turret_medium","Hpt_multicannon_turret_small","Hpt_plasmaaccelerator_fixed_large","Hpt_plasmaaccelerator_fixed_medium","Hpt_plasmapointdefence_turret_tiny","Hpt_pulselaser_fixed_huge","Hpt_pulselaser_fixed_medium","Hpt_pulselaser_fixed_small","Hpt_pulselaser_gimbal_huge","Hpt_pulselaser_gimbal_medium","Hpt_pulselaser_gimbal_small","Hpt_pulselaser_turret_large","Hpt_pulselaser_turret_medium","Hpt_pulselaser_turret_small","Hpt_pulselaserburst_fixed_large","Hpt_pulselaserburst_fixed_medium","Hpt_pulselaserburst_fixed_small","Hpt_pulselaserburst_gimbal_huge","Hpt_pulselaserburst_gimbal_large","Hpt_pulselaserburst_gimbal_small","Hpt_pulselaserburst_turret_large","Hpt_pulselaserburst_turret_medium","Hpt_railgun_fixed_medium","Hpt_railgun_fixed_small","Hpt_shieldbooster_size0_class1","Hpt_shieldbooster_size0_class2","Hpt_shieldbooster_size0_class3","Hpt_shieldbooster_size0_class4","Hpt_shieldbooster_size0_class5","Hpt_slugshot_fixed_large","Hpt_slugshot_fixed_medium","Hpt_slugshot_fixed_small","Hpt_slugshot_gimbal_medium","Hpt_slugshot_gimbal_small","Hpt_slugshot_turret_large","Hpt_slugshot_turret_small","Int_buggybay_size2_class1","Int_buggybay_size2_class2","Int_buggybay_size4_class1","Int_buggybay_size4_class2","Int_buggybay_size6_class1","Int_buggybay_size6_class2","Int_cargorack_size1_class1","Int_cargorack_size2_class1","Int_cargorack_size3_class1","Int_cargorack_size4_class1","Int_cargorack_size5_class1","Int_cargorack_size6_class1","Int_cargorack_size7_class1","Int_cargorack_size8_class1","Int_detailedsurfacescanner_tiny","Int_dockingcomputer_standard","Int_dronecontrol_collection_size1_class1","Int_dronecontrol_collection_size1_class2","Int_dronecontrol_collection_size1_class3","Int_dronecontrol_collection_size1_class4","Int_dronecontrol_collection_size1_class5","Int_dronecontrol_collection_size3_class1","Int_dronecontrol_collection_size3_class2","Int_dronecontrol_collection_size3_class3","Int_dronecontrol_collection_size3_class4","Int_dronecontrol_collection_size3_class5","Int_dronecontrol_collection_size5_class1","Int_dronecontrol_collection_size5_class2","Int_dronecontrol_collection_size5_class3","Int_dronecontrol_collection_size5_class4","Int_dronecontrol_collection_size5_class5","Int_dronecontrol_collection_size7_class1","Int_dronecontrol_collection_size7_class2","Int_dronecontrol_collection_size7_class4","Int_dronecontrol_collection_size7_class5","Int_dronecontrol_fueltransfer_size1_class1","Int_dronecontrol_fueltransfer_size1_class2","Int_dronecontrol_fueltransfer_size1_class3","Int_dronecontrol_fueltransfer_size1_class4","Int_dronecontrol_fueltransfer_size1_class5","Int_dronecontrol_fueltransfer_size3_class1","Int_dronecontrol_fueltransfer_size3_class2","Int_dronecontrol_fueltransfer_size3_class3","Int_dronecontrol_fueltransfer_size3_class4","Int_dronecontrol_fueltransfer_size5_class1","Int_dronecontrol_fueltransfer_size5_class2","Int_dronecontrol_fueltransfer_size5_class3","Int_dronecontrol_fueltransfer_size5_class4","Int_dronecontrol_fueltransfer_size7_class1","Int_dronecontrol_fueltransfer_size7_class2","Int_dronecontrol_fueltransfer_size7_class3","Int_dronecontrol_fueltransfer_size7_class4","Int_dronecontrol_fueltransfer_size7_class5","Int_dronecontrol_prospector_size1_class1","Int_dronecontrol_prospector_size1_class2","Int_dronecontrol_prospector_size1_class3","Int_dronecontrol_prospector_size1_class4","Int_dronecontrol_prospector_size1_class5","Int_dronecontrol_prospector_size3_class1","Int_dronecontrol_prospector_size3_class2","Int_dronecontrol_prospector_size3_class3","Int_dronecontrol_prospector_size3_class4","Int_dronecontrol_prospector_size3_class5","Int_dronecontrol_prospector_size5_class1","Int_dronecontrol_prospector_size5_class2","Int_dronecontrol_prospector_size5_class3","Int_dronecontrol_prospector_size5_class4","Int_dronecontrol_prospector_size7_class1","Int_dronecontrol_prospector_size7_class2","Int_dronecontrol_prospector_size7_class3","Int_dronecontrol_prospector_size7_class4","Int_dronecontrol_recon_size1_class1","Int_dronecontrol_recon_size3_class1","Int_dronecontrol_recon_size5_class1","Int_dronecontrol_recon_size7_class1","Int_dronecontrol_repair_size1_class1","Int_dronecontrol_repair_size1_class2","Int_dronecontrol_repair_size1_class3","Int_dronecontrol_repair_size1_class4","Int_dronecontrol_repair_size1_class5","Int_dronecontrol_repair_size3_class1","Int_dronecontrol_repair_size3_class2","Int_dronecontrol_repair_size3_class3","Int_dronecontrol_repair_size3_class5","Int_dronecontrol_repair_size5_class1","Int_dronecontrol_repair_size5_class2","Int_dronecontrol_repair_size5_class3","Int_dronecontrol_repair_size5_class4","Int_dronecontrol_repair_size5_class5","Int_dronecontrol_repair_size7_class1","Int_dronecontrol_repair_size7_class2","Int_dronecontrol_repair_size7_class3","Int_dronecontrol_repair_size7_class4","Int_dronecontrol_resourcesiphon_size1_class1","Int_dronecontrol_resourcesiphon_size1_class2","Int_dronecontrol_resourcesiphon_size1_class3","Int_dronecontrol_resourcesiphon_size1_class4","Int_dronecontrol_resourcesiphon_size1_class5","Int_dronecontrol_resourcesiphon_size3_class1","Int_dronecontrol_resourcesiphon_size3_class2","Int_dronecontrol_resourcesiphon_size3_class3","Int_dronecontrol_resourcesiphon_size3_class4","Int_dronecontrol_resourcesiphon_size3_class5","Int_dronecontrol_resourcesiphon_size5_class1","Int_dronecontrol_resourcesiphon_size5_class2","Int_dronecontrol_resourcesiphon_size5_class3","Int_dronecontrol_resourcesiphon_size5_class4","Int_dronecontrol_resourcesiphon_size5_class5","Int_dronecontrol_resourcesiphon_size7_class1","Int_dronecontrol_resourcesiphon_size7_class2","Int_dronecontrol_resourcesiphon_size7_class3","Int_dronecontrol_resourcesiphon_size7_class4","Int_engine_size2_class1","Int_engine_size2_class2","Int_engine_size2_class3","Int_engine_size2_class4","Int_engine_size2_class5","Int_engine_size3_class1","Int_engine_size3_class2","Int_engine_size3_class3","Int_engine_size3_class4","Int_engine_size3_class5","Int_engine_size4_class1","Int_engine_size4_class2","Int_engine_size4_class3","Int_engine_size4_class4","Int_engine_size4_class5","Int_engine_size5_class1","Int_engine_size5_class2","Int_engine_size5_class3","Int_engine_size5_class4","Int_engine_size5_class5","Int_engine_size6_class1","Int_engine_size6_class2","Int_engine_size6_class3","Int_engine_size6_class4","Int_engine_size6_class5","Int_engine_size7_class1","Int_engine_size7_class2","Int_engine_size7_class3","Int_engine_size8_class1","Int_engine_size8_class2","Int_engine_size8_class5","Int_fighterbay_size5_class1","Int_fighterbay_size6_class1","Int_fighterbay_size7_class1","Int_fsdinterdictor_size1_class1","Int_fsdinterdictor_size1_class2","Int_fsdinterdictor_size1_class3","Int_fsdinterdictor_size1_class4","Int_fsdinterdictor_size1_class5","Int_fsdinterdictor_size2_class1","Int_fsdinterdictor_size2_class2","Int_fsdinterdictor_size2_class3","Int_fsdinterdictor_size2_class4","Int_fsdinterdictor_size2_class5","Int_fsdinterdictor_size3_class1","Int_fsdinterdictor_size3_class2","Int_fsdinterdictor_size3_class3","Int_fsdinterdictor_size3_class4","Int_fsdinterdictor_size4_class1","Int_fsdinterdictor_size4_class2","Int_fsdinterdictor_size4_class3","Int_fsdinterdictor_size4_class4","Int_fsdinterdictor_size4_class5","Int_fuelscoop_size1_class1","Int_fuelscoop_size1_class2","Int_fuelscoop_size1_class3","Int_fuelscoop_size1_class4","Int_fuelscoop_size2_class1","Int_fuelscoop_size2_class2","Int_fuelscoop_size2_class3","Int_fuelscoop_size2_class4","Int_fuelscoop_size2_class5","Int_fuelscoop_size3_class1","Int_fuelscoop_size3_class2","Int_fuelscoop_size3_class3","Int_fuelscoop_size7_class1","Int_fuelscoop_size7_class2","Int_fuelscoop_size7_class4","Int_fuelscoop_size8_class1","Int_fuelscoop_size8_class2","Int_fuelscoop_size8_class3","Int_fuelscoop_size8_class4","Int_fueltank_size1_class3","Int_fueltank_size2_class3","Int_fueltank_size3_class3","Int_fueltank_size4_class3","Int_fueltank_size5_class3","Int_fueltank_size6_class3","Int_fueltank_size7_class3","Int_fueltank_size8_class3","Int_hullreinforcement_size1_class1","Int_hullreinforcement_size1_class2","Int_hullreinforcement_size2_class1","Int_hullreinforcement_size2_class2","Int_hullreinforcement_size3_class1","Int_hullreinforcement_size3_class2","Int_hullreinforcement_size4_class1","Int_hullreinforcement_size4_class2","Int_hullreinforcement_size5_class1","Int_hullreinforcement_size5_class2","Int_hyperdrive_overcharge_size2_class1","Int_hyperdrive_overcharge_size2_class2","Int_hyperdrive_overcharge_size2_class3","Int_hyperdrive_overcharge_size2_class4","Int_hyperdrive_overcharge_size2_class5","Int_hyperdrive_overcharge_size3_class1","Int_hyperdrive_overcharge_size3_class2","Int_hyperdrive_overcharge_size3_class3","Int_hyperdrive_overcharge_size3_class4","Int_hyperdrive_overcharge_size3_class5","Int_hyperdrive_overcharge_size4_class1","Int_hyperdrive_overcharge_size4_class2","Int_hyperdrive_overcharge_size4_class3","Int_hyperdrive_overcharge_size4_class4","Int_hyperdrive_overcharge_size4_class5","Int_hyperdrive_overcharge_size5_class1","Int_hyperdrive_overcharge_size5_class2","Int_hyperdrive_overcharge_size5_class3","Int_hyperdrive_overcharge_size5_class4","Int_hyperdrive_overcharge_size5_class5","Int_hyperdrive_overcharge_size6_class1","Int_hyperdrive_overcharge_size6_class2","Int_hyperdrive_overcharge_size6_class3","Int_hyperdrive_overcharge_size6_class4","Int_hyperdrive_overcharge_size6_class5","Int_hyperdrive_overcharge_size7_class1","Int_hyperdrive_overcharge_size7_class2","Int_hyperdrive_overcharge_size7_class3","Int_hyperdrive_overcharge_size7_class4","Int_hyperdrive_overcharge_size7_class5","Int_hyperdrive_size2_class1","Int_hyperdrive_size2_class2","Int_hyperdrive_size2_class3","Int_hyperdrive_size2_class4","Int_hyperdrive_size2_class5","Int_hyperdrive_size3_class1","Int_hyperdrive_size3_class2","Int_hyperdrive_size3_class3","Int_hyperdrive_size3_class4","Int_hyperdrive_size3_class5","Int_hyperdrive_size4_class1","Int_hyperdrive_size4_class2","Int_hyperdrive_size4_class3","Int_hyperdrive_size4_class4","Int_hyperdrive_size4_class5","Int_hyperdrive_size5_class1","Int_hyperdrive_size5_class2","Int_hyperdrive_size5_class3","Int_hyperdrive_size5_class4","Int_hyperdrive_size5_class5","Int_hyperdrive_size6_class1","Int_hyperdrive_size6_class2","Int_hyperdrive_size6_class3","Int_hyperdrive_size6_class4","Int_hyperdrive_size7_class1","Int_hyperdrive_size7_class3","Int_largecargorack_size7_class1","Int_largecargorack_size8_class1","Int_lifesupport_size1_class1","Int_lifesupport_size1_class2","Int_lifesupport_size1_class3","Int_lifesupport_size1_class4","Int_lifesupport_size1_class5","Int_lifesupport_size2_class1","Int_lifesupport_size2_class2","Int_lifesupport_size2_class3","Int_lifesupport_size2_class4","Int_lifesupport_size2_class5","Int_lifesupport_size3_class1","Int_lifesupport_size3_class2","Int_lifesupport_size3_class3","Int_lifesupport_size3_class4","Int_lifesupport_size4_class1","Int_lifesupport_size4_class2","Int_lifesupport_size4_class3","Int_lifesupport_size4_class4","Int_lifesupport_size4_class5","Int_lifesupport_size5_class1","Int_lifesupport_size5_class2","Int_lifesupport_size5_class3","Int_lifesupport_size5_class4","Int_lifesupport_size5_class5","Int_lifesupport_size6_class1","Int_lifesupport_size6_class2","Int_lifesupport_size6_class3","Int_lifesupport_size6_class4","Int_lifesupport_size6_class5","Int_lifesupport_size7_class1","Int_lifesupport_size7_class2","Int_lifesupport_size7_class3","Int_lifesupport_size7_class4","Int_lifesupport_size7_class5","Int_lifesupport_size8_class1","Int_lifesupport_size8_class4","Int_lifesupport_size8_class5","Int_modulereinforcement_size1_class1","Int_modulereinforcement_size1_class2","Int_modulereinforcement_size2_class1","Int_modulereinforcement_size3_class1","Int_modulereinforcement_size3_class2","Int_modulereinforcement_size4_class1","Int_modulereinforcement_size4_class2","Int_modulereinforcement_size5_class1","Int_modulereinforcement_size5_class2","Int_multidronecontrol_mining_size3_class1","Int_multidronecontrol_mining_size3_class3","Int_multidronecontrol_operations_size3_class3","Int_multidronecontrol_operations_size3_class4","Int_multidronecontrol_rescue_size3_class2","Int_multidronecontrol_rescue_size3_class3","Int_multidronecontrol_xeno_size3_class3","Int_multidronecontrol_xeno_size3_class4","Int_passengercabin_size2_class1","Int_passengercabin_size3_class2","Int_passengercabin_size4_class1","Int_passengercabin_size4_class2","Int_passengercabin_size4_class3","Int_passengercabin_size5_class1","Int_passengercabin_size5_class2","Int_passengercabin_size5_class3","Int_passengercabin_size6_class1","Int_passengercabin_size6_class2","Int_passengercabin_size6_class3","Int_passengercabin_size6_class4","Int_powerdistributor_size1_class1","Int_powerdistributor_size1_class2","Int_powerdistributor_size1_class3","Int_powerdistributor_size1_class5","Int_powerdistributor_size2_class1","Int_powerdistributor_size2_class2","Int_powerdistributor_size2_class3","Int_powerdistributor_size2_class4","Int_powerdistributor_size2_class5","Int_powerdistributor_size3_class1","Int_powerdistributor_size3_class2","Int_powerdistributor_size3_class3","Int_powerdistributor_size3_class4","Int_powerdistributor_size3_class5","Int_powerdistributor_size4_class1","Int_powerdistributor_size4_class2","Int_powerdistributor_size4_class3","Int_powerdistributor_size4_class5","Int_powerdistributor_size5_class1","Int_powerdistributor_size5_class2","Int_powerdistributor_size5_class3","Int_powerdistributor_size5_class4","Int_powerdistributor_size5_class5","Int_powerdistributor_size6_class1","Int_powerdistributor_size6_class2","Int_powerdistributor_size6_class3","Int_powerdistributor_size6_class4","Int_powerdistributor_size7_class1","Int_powerdistributor_size7_class2","Int_powerdistributor_size7_class3","Int_powerdistributor_size7_class4","Int_powerdistributor_size7_class5","Int_powerdistributor_size8_class1","Int_powerdistributor_size8_class2","Int_powerdistributor_size8_class4","Int_powerdistributor_size8_class5","Int_powerplant_size2_class1","Int_powerplant_size2_class2","Int_powerplant_size2_class3","Int_powerplant_size2_class4","Int_powerplant_size2_class5","Int_powerplant_size3_class1","Int_powerplant_size3_class2","Int_powerplant_size3_class3","Int_powerplant_size3_class4","Int_powerplant_size3_class5","Int_powerplant_size4_class1","Int_powerplant_size4_class2","Int_powerplant_size4_class3","Int_powerplant_size4_class4","Int_powerplant_size4_class5","Int_powerplant_size5_class1","Int_powerplant_size5_class2","Int_powerplant_size5_class3","Int_powerplant_size5_class4","Int_powerplant_size5_class5","Int_powerplant_size6_class1","Int_powerplant_size6_class2","Int_powerplant_size6_class3","Int_powerplant_size6_class4","Int_powerplant_size7_class1","Int_powerplant_size7_class2","Int_powerplant_size7_class3","Int_powerplant_size7_class5","Int_powerplant_size8_class1","Int_powerplant_size8_class2","Int_powerplant_size8_class3","Int_refinery_size1_class1","Int_refinery_size1_class2","Int_refinery_size1_class3","Int_refinery_size1_class4","Int_refinery_size1_class5","Int_refinery_size2_class1","Int_refinery_size2_class2","Int_refinery_size2_class3","Int_refinery_size2_class4","Int_refinery_size2_class5","Int_refinery_size3_class1","Int_refinery_size3_class2","Int_refinery_size3_class3","Int_refinery_size3_class4","Int_refinery_size3_class5","Int_refinery_size4_class1","Int_refinery_size4_class2","Int_refinery_size4_class3","Int_refinery_size4_class4","Int_repairer_size1_class1","Int_repairer_size1_class2","Int_repairer_size1_class3","Int_repairer_size1_class4","Int_repairer_size1_class5","Int_repairer_size2_class1","Int_repairer_size2_class2","Int_repairer_size2_class3","Int_repairer_size2_class4","Int_repairer_size2_class5","Int_repairer_size3_class1","Int_repairer_size3_class2","Int_repairer_size3_class3","Int_repairer_size3_class4","Int_repairer_size3_class5","Int_repairer_size4_class1","Int_repairer_size4_class2","Int_repairer_size4_class3","Int_repairer_size4_class4","Int_repairer_size4_class5","Int_repairer_size5_class1","Int_repairer_size5_class2","Int_repairer_size5_class3","Int_repairer_size5_class4","Int_repairer_size6_class1","Int_repairer_size6_class2","Int_repairer_size6_class3","Int_repairer_size6_class4","Int_repairer_size7_class1","Int_repairer_size7_class2","Int_repairer_size7_class3","Int_repairer_size8_class1","Int_repairer_size8_class2","Int_repairer_size8_class3","Int_repairer_size8_class4","Int_sensors_size1_class1","Int_sensors_size1_class2","Int_sensors_size1_class3","Int_sensors_size1_class4","Int_sensors_size1_class5","Int_sensors_size2_class1","Int_sensors_size2_class2","Int_sensors_size2_class3","Int_sensors_size2_class4","Int_sensors_size2_class5","Int_sensors_size3_class1","Int_sensors_size3_class2","Int_sensors_size3_class3","Int_sensors_size3_class4","Int_sensors_size3_class5","Int_sensors_size4_class1","Int_sensors_size4_class2","Int_sensors_size4_class3","Int_sensors_size4_class4","Int_sensors_size4_class5","Int_sensors_size5_class1","Int_sensors_size5_class2","Int_sensors_size5_class3","Int_sensors_size5_class4","Int_sensors_size6_class1","Int_sensors_size6_class2","Int_sensors_size6_class3","Int_sensors_size6_class4","Int_sensors_size7_class1","Int_sensors_size7_class2","Int_sensors_size7_class3","Int_sensors_size7_class4","Int_sensors_size7_class5","Int_sensors_size8_class1","Int_sensors_size8_class2","Int_sensors_size8_class3","Int_sensors_size8_class5","Int_shieldcellbank_size1_class1","Int_shieldcellbank_size1_class2","Int_shieldcellbank_size1_class3","Int_shieldcellbank_size1_class4","Int_shieldcellbank_size1_class5","Int_shieldcellbank_size2_class1","Int_shieldcellbank_size2_class2","Int_shieldcellbank_size2_class3","Int_shieldcellbank_size2_class5","Int_shieldcellbank_size3_class1","Int_shieldcellbank_size3_class2","Int_shieldcellbank_size3_class3","Int_shieldcellbank_size3_class4","Int_shieldcellbank_size3_class5","Int_shieldcellbank_size4_class1","Int_shieldcellbank_size4_class2","Int_shieldcellbank_size4_class3","Int_shieldcellbank_size4_class4","Int_shieldcellbank_size4_class5","Int_shieldcellbank_size5_class1","Int_shieldcellbank_size5_class2","Int_shieldcellbank_size5_class4","Int_shieldcellbank_size6_class1","Int_shieldcellbank_size6_class2","Int_shieldcellbank_size6_class3","Int_shieldcellbank_size6_class4","Int_shieldcellbank_size6_class5","Int_shieldcellbank_size7_class1","Int_shieldcellbank_size7_class2","Int_shieldcellbank_size7_class3","Int_shieldcellbank_size7_class4","Int_shieldcellbank_size7_class5","Int_shieldcellbank_size8_class1","Int_shieldcellbank_size8_class3","Int_shieldcellbank_size8_class4","Int_shieldgenerator_size1_class3_fast","Int_shieldgenerator_size2_class1","Int_shieldgenerator_size2_class2","Int_shieldgenerator_size2_class3","Int_shieldgenerator_size2_class3_fast","Int_shieldgenerator_size2_class4","Int_shieldgenerator_size2_class5","Int_shieldgenerator_size3_class1","Int_shieldgenerator_size3_class2","Int_shieldgenerator_size3_class3","Int_shieldgenerator_size3_class3_fast","Int_shieldgenerator_size3_class4","Int_shieldgenerator_size3_class5","Int_shieldgenerator_size4_class1","Int_shieldgenerator_size4_class2","Int_shieldgenerator_size4_class3","Int_shieldgenerator_size4_class3_fast","Int_shieldgenerator_size4_class4","Int_shieldgenerator_size4_class5","Int_shieldgenerator_size5_class1","Int_shieldgenerator_size5_class2","Int_shieldgenerator_size5_class3","Int_shieldgenerator_size5_class3_fast","Int_shieldgenerator_size5_class4","Int_shieldgenerator_size5_class5","Int_shieldgenerator_size6_class1","Int_shieldgenerator_size6_class2","Int_shieldgenerator_size6_class3","Int_shieldgenerator_size6_class3_fast","Int_shieldgenerator_size6_class4","Int_shieldgenerator_size6_class5","Int_shieldgenerator_size7_class1","Int_shieldgenerator_size7_class2","Int_shieldgenerator_size7_class3_fast","Int_shieldgenerator_size7_class4","Int_shieldgenerator_size7_class5","Int_shieldgenerator_size8_class1","Int_shieldgenerator_size8_class2","Int_shieldgenerator_size8_class3","Int_shieldgenerator_size8_class3_fast","Int_shieldgenerator_size8_class4","Int_shieldgenerator_size8_class5","Int_supercruiseassist","anaconda_Armour_grade1","anaconda_Armour_grade2","anaconda_Armour_grade3","anaconda_Armour_mirrored","anaconda_Armour_reactive","asp_Armour_grade1","asp_Armour_grade2","asp_Armour_grade3","asp_Armour_mirrored","asp_Armour_reactive","asp_scout_Armour_grade1","asp_scout_Armour_grade2","asp_scout_Armour_grade3","asp_scout_Armour_mirrored","asp_scout_Armour_reactive","belugaliner_Armour_grade1","belugaliner_Armour_grade2","belugaliner_Armour_grade3","belugaliner_Armour_mirrored","belugaliner_Armour_reactive","cobramkiii_Armour_grade1","cobramkiii_Armour_grade2","cobramkiii_Armour_grade3","cobramkiii_Armour_mirrored","cobramkiii_Armour_reactive","cobramkiv_Armour_grade1","cobramkiv_Armour_grade2","cobramkiv_Armour_grade3","cobramkiv_Armour_mirrored","cobramkiv_Armour_reactive","diamondback_Armour_grade1","diamondback_Armour_grade2","diamondback_Armour_grade3","diamondback_Armour_mirrored","diamondback_Armour_reactive","diamondbackxl_Armour_grade1","diamondbackxl_Armour_grade2","diamondbackxl_Armour_grade3","diamondbackxl_Armour_mirrored","diamondbackxl_Armour_reactive","dolphin_Armour_grade1","dolphin_Armour_grade2","dolphin_Armour_grade3","dolphin_Armour_mirrored","dolphin_Armour_reactive","eagle_Armour_grade1","eagle_Armour_grade2","eagle_Armour_grade3","eagle_Armour_mirrored","eagle_Armour_reactive","ferdelance_Armour_grade1","ferdelance_Armour_grade2","ferdelance_Armour_grade3","ferdelance_Armour_mirrored","ferdelance_Armour_reactive","hauler_Armour_grade1","hauler_Armour_grade2","hauler_Armour_grade3","hauler_Armour_mirrored","hauler_Armour_reactive","independant_trader_Armour_grade1","independant_trader_Armour_grade2","independant_trader_Armour_grade3","independant_trader_Armour_mirrored","independant_trader_Armour_reactive","krait_light_Armour_grade1","krait_light_Armour_grade2","krait_light_Armour_grade3","krait_light_Armour_mirrored","krait_light_Armour_reactive","krait_mkii_Armour_grade1","krait_mkii_Armour_grade2","krait_mkii_Armour_grade3","krait_mkii_Armour_mirrored","krait_mkii_Armour_reactive","mamba_Armour_grade1","mamba_Armour_grade2","mamba_Armour_grade3","mamba_Armour_mirrored","mamba_Armour_reactive","python_Armour_grade1","python_Armour_grade2","python_Armour_grade3","python_Armour_mirrored","python_Armour_reactive","sidewinder_Armour_grade1","sidewinder_Armour_grade2","sidewinder_Armour_grade3","sidewinder_Armour_mirrored","sidewinder_Armour_reactive","type6_Armour_grade1","type6_Armour_grade2","type6_Armour_grade3","type6_Armour_mirrored","type6_Armour_reactive","type7_Armour_grade1","type7_Armour_grade2","type7_Armour_grade3","type7_Armour_mirrored","type7_Armour_reactive","type9_Armour_grade1","type9_Armour_grade2","type9_Armour_grade3","type9_Armour_mirrored","type9_Armour_reactive","type9_military_Armour_grade1","type9_military_Armour_grade2","typex_2_Armour_grade1","typex_2_Armour_grade2","typex_2_Armour_grade3","typex_3_Armour_grade1","typex_3_Armour_grade2","typex_3_Armour_grade3","viper_Armour_grade1","viper_Armour_grade2","viper_Armour_grade3","viper_Armour_mirrored","viper_Armour_reactive","viper_mkiv_Armour_grade1","viper_mkiv_Armour_grade2","viper_mkiv_Armour_grade3","viper_mkiv_Armour_mirrored","viper_mkiv_Armour_reactive","vulture_Armour_grade1","vulture_Armour_grade2","vulture_Armour_grade3","vulture_Armour_mirrored","vulture_Armour_reactive"],"odyssey":true,"stationName":"Soundand Vision","systemName":"Delta-2 Chamaelontis","timestamp":"2025-09-08T14:50:10Z"}
            let market_id = match json["marketId"].as_i64() {
                None => {
                    return Err(EdcasError {
                        0: "modules: No market id".to_string(),
                    });
                }
                Some(value) => value,
            };
            match client.execute(
                //language=postgresql
                "DELETE FROM modul_listening WHERE market_id=$1",
                &[&market_id],
            ) {
                Ok(_) => {}
                Err(err) => {
                    error!(
                        "[{}] Couldn't delete old modul listenings: {}",
                        journal_id, err
                    );
                }
            }
            let modul_size = json["modules"].len();
            for i in 0..modul_size {
                let modul_name = value_table(
                    Tables::ModulName,
                    json["modules"][i].to_string(),
                    journal_id,
                    client,
                )?;
                match client.execute(
                    //language=postgresql
                    "INSERT INTO modul_listening (modul_name, market_id, journal_id) VALUES ($1,$2,$3)",
                    &[&modul_name,&market_id,&journal_id]
                ) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("[{}] Couldn't insert modul listening: {}",journal_id,err);
                    }
                }
            }
            return Ok(());
        }
        "marketId" => {
            //{"horizons":true,"marketId":4253567491,"modules":["Hpt_advancedtorppylon_fixed_large","Hpt_advancedtorppylon_fixed_medium","Hpt_advancedtorppylon_fixed_small","Hpt_basicmissilerack_fixed_large","Hpt_basicmissilerack_fixed_medium","Hpt_basicmissilerack_fixed_small","Hpt_beamlaser_fixed_huge","Hpt_beamlaser_fixed_large","Hpt_beamlaser_fixed_medium","Hpt_beamlaser_fixed_small","Hpt_beamlaser_gimbal_huge","Hpt_beamlaser_gimbal_large","Hpt_beamlaser_gimbal_medium","Hpt_beamlaser_gimbal_small","Hpt_beamlaser_turret_small","Hpt_cannon_fixed_huge","Hpt_cannon_fixed_large","Hpt_cannon_fixed_medium","Hpt_cannon_gimbal_huge","Hpt_cannon_gimbal_large","Hpt_cannon_gimbal_medium","Hpt_cannon_gimbal_small","Hpt_cannon_turret_large","Hpt_cannon_turret_medium","Hpt_cannon_turret_small","Hpt_cargoscanner_size0_class1","Hpt_cargoscanner_size0_class2","Hpt_cargoscanner_size0_class3","Hpt_cargoscanner_size0_class4","Hpt_cargoscanner_size0_class5","Hpt_chafflauncher_tiny","Hpt_cloudscanner_size0_class1","Hpt_cloudscanner_size0_class2","Hpt_cloudscanner_size0_class3","Hpt_cloudscanner_size0_class4","Hpt_cloudscanner_size0_class5","Hpt_crimescanner_size0_class1","Hpt_crimescanner_size0_class2","Hpt_crimescanner_size0_class3","Hpt_crimescanner_size0_class4","Hpt_crimescanner_size0_class5","Hpt_dumbfiremissilerack_fixed_large","Hpt_dumbfiremissilerack_fixed_medium","Hpt_dumbfiremissilerack_fixed_small","Hpt_electroniccountermeasure_tiny","Hpt_heatsinklauncher_turret_tiny","Hpt_minelauncher_fixed_small","Hpt_minelauncher_fixed_small_impulse","Hpt_mining_abrblstr_fixed_small","Hpt_mining_seismchrgwarhd_fixed_medium","Hpt_mining_subsurfdispmisle_fixed_medium","Hpt_mining_subsurfdispmisle_fixed_small","Hpt_mining_subsurfdispmisle_turret_medium","Hpt_mininglaser_turret_small","Hpt_mrascanner_size0_class1","Hpt_mrascanner_size0_class2","Hpt_mrascanner_size0_class5","Hpt_multicannon_fixed_huge","Hpt_multicannon_fixed_large","Hpt_multicannon_fixed_medium","Hpt_multicannon_fixed_small","Hpt_multicannon_gimbal_huge","Hpt_multicannon_gimbal_large","Hpt_multicannon_gimbal_medium","Hpt_multicannon_gimbal_small","Hpt_multicannon_turret_medium","Hpt_multicannon_turret_small","Hpt_plasmaaccelerator_fixed_large","Hpt_plasmaaccelerator_fixed_medium","Hpt_plasmapointdefence_turret_tiny","Hpt_pulselaser_fixed_huge","Hpt_pulselaser_fixed_medium","Hpt_pulselaser_fixed_small","Hpt_pulselaser_gimbal_huge","Hpt_pulselaser_gimbal_medium","Hpt_pulselaser_gimbal_small","Hpt_pulselaser_turret_large","Hpt_pulselaser_turret_medium","Hpt_pulselaser_turret_small","Hpt_pulselaserburst_fixed_large","Hpt_pulselaserburst_fixed_medium","Hpt_pulselaserburst_fixed_small","Hpt_pulselaserburst_gimbal_huge","Hpt_pulselaserburst_gimbal_large","Hpt_pulselaserburst_gimbal_small","Hpt_pulselaserburst_turret_large","Hpt_pulselaserburst_turret_medium","Hpt_railgun_fixed_medium","Hpt_railgun_fixed_small","Hpt_shieldbooster_size0_class1","Hpt_shieldbooster_size0_class2","Hpt_shieldbooster_size0_class3","Hpt_shieldbooster_size0_class4","Hpt_shieldbooster_size0_class5","Hpt_slugshot_fixed_large","Hpt_slugshot_fixed_medium","Hpt_slugshot_fixed_small","Hpt_slugshot_gimbal_medium","Hpt_slugshot_gimbal_small","Hpt_slugshot_turret_large","Hpt_slugshot_turret_small","Int_buggybay_size2_class1","Int_buggybay_size2_class2","Int_buggybay_size4_class1","Int_buggybay_size4_class2","Int_buggybay_size6_class1","Int_buggybay_size6_class2","Int_cargorack_size1_class1","Int_cargorack_size2_class1","Int_cargorack_size3_class1","Int_cargorack_size4_class1","Int_cargorack_size5_class1","Int_cargorack_size6_class1","Int_cargorack_size7_class1","Int_cargorack_size8_class1","Int_detailedsurfacescanner_tiny","Int_dockingcomputer_standard","Int_dronecontrol_collection_size1_class1","Int_dronecontrol_collection_size1_class2","Int_dronecontrol_collection_size1_class3","Int_dronecontrol_collection_size1_class4","Int_dronecontrol_collection_size1_class5","Int_dronecontrol_collection_size3_class1","Int_dronecontrol_collection_size3_class2","Int_dronecontrol_collection_size3_class3","Int_dronecontrol_collection_size3_class4","Int_dronecontrol_collection_size3_class5","Int_dronecontrol_collection_size5_class1","Int_dronecontrol_collection_size5_class2","Int_dronecontrol_collection_size5_class3","Int_dronecontrol_collection_size5_class4","Int_dronecontrol_collection_size5_class5","Int_dronecontrol_collection_size7_class1","Int_dronecontrol_collection_size7_class2","Int_dronecontrol_collection_size7_class4","Int_dronecontrol_collection_size7_class5","Int_dronecontrol_fueltransfer_size1_class1","Int_dronecontrol_fueltransfer_size1_class2","Int_dronecontrol_fueltransfer_size1_class3","Int_dronecontrol_fueltransfer_size1_class4","Int_dronecontrol_fueltransfer_size1_class5","Int_dronecontrol_fueltransfer_size3_class1","Int_dronecontrol_fueltransfer_size3_class2","Int_dronecontrol_fueltransfer_size3_class3","Int_dronecontrol_fueltransfer_size3_class4","Int_dronecontrol_fueltransfer_size5_class1","Int_dronecontrol_fueltransfer_size5_class2","Int_dronecontrol_fueltransfer_size5_class3","Int_dronecontrol_fueltransfer_size5_class4","Int_dronecontrol_fueltransfer_size7_class1","Int_dronecontrol_fueltransfer_size7_class2","Int_dronecontrol_fueltransfer_size7_class3","Int_dronecontrol_fueltransfer_size7_class4","Int_dronecontrol_fueltransfer_size7_class5","Int_dronecontrol_prospector_size1_class1","Int_dronecontrol_prospector_size1_class2","Int_dronecontrol_prospector_size1_class3","Int_dronecontrol_prospector_size1_class4","Int_dronecontrol_prospector_size1_class5","Int_dronecontrol_prospector_size3_class1","Int_dronecontrol_prospector_size3_class2","Int_dronecontrol_prospector_size3_class3","Int_dronecontrol_prospector_size3_class4","Int_dronecontrol_prospector_size3_class5","Int_dronecontrol_prospector_size5_class1","Int_dronecontrol_prospector_size5_class2","Int_dronecontrol_prospector_size5_class3","Int_dronecontrol_prospector_size5_class4","Int_dronecontrol_prospector_size7_class1","Int_dronecontrol_prospector_size7_class2","Int_dronecontrol_prospector_size7_class3","Int_dronecontrol_prospector_size7_class4","Int_dronecontrol_recon_size1_class1","Int_dronecontrol_recon_size3_class1","Int_dronecontrol_recon_size5_class1","Int_dronecontrol_recon_size7_class1","Int_dronecontrol_repair_size1_class1","Int_dronecontrol_repair_size1_class2","Int_dronecontrol_repair_size1_class3","Int_dronecontrol_repair_size1_class4","Int_dronecontrol_repair_size1_class5","Int_dronecontrol_repair_size3_class1","Int_dronecontrol_repair_size3_class2","Int_dronecontrol_repair_size3_class3","Int_dronecontrol_repair_size3_class5","Int_dronecontrol_repair_size5_class1","Int_dronecontrol_repair_size5_class2","Int_dronecontrol_repair_size5_class3","Int_dronecontrol_repair_size5_class4","Int_dronecontrol_repair_size5_class5","Int_dronecontrol_repair_size7_class1","Int_dronecontrol_repair_size7_class2","Int_dronecontrol_repair_size7_class3","Int_dronecontrol_repair_size7_class4","Int_dronecontrol_resourcesiphon_size1_class1","Int_dronecontrol_resourcesiphon_size1_class2","Int_dronecontrol_resourcesiphon_size1_class3","Int_dronecontrol_resourcesiphon_size1_class4","Int_dronecontrol_resourcesiphon_size1_class5","Int_dronecontrol_resourcesiphon_size3_class1","Int_dronecontrol_resourcesiphon_size3_class2","Int_dronecontrol_resourcesiphon_size3_class3","Int_dronecontrol_resourcesiphon_size3_class4","Int_dronecontrol_resourcesiphon_size3_class5","Int_dronecontrol_resourcesiphon_size5_class1","Int_dronecontrol_resourcesiphon_size5_class2","Int_dronecontrol_resourcesiphon_size5_class3","Int_dronecontrol_resourcesiphon_size5_class4","Int_dronecontrol_resourcesiphon_size5_class5","Int_dronecontrol_resourcesiphon_size7_class1","Int_dronecontrol_resourcesiphon_size7_class2","Int_dronecontrol_resourcesiphon_size7_class3","Int_dronecontrol_resourcesiphon_size7_class4","Int_engine_size2_class1","Int_engine_size2_class2","Int_engine_size2_class3","Int_engine_size2_class4","Int_engine_size2_class5","Int_engine_size3_class1","Int_engine_size3_class2","Int_engine_size3_class3","Int_engine_size3_class4","Int_engine_size3_class5","Int_engine_size4_class1","Int_engine_size4_class2","Int_engine_size4_class3","Int_engine_size4_class4","Int_engine_size4_class5","Int_engine_size5_class1","Int_engine_size5_class2","Int_engine_size5_class3","Int_engine_size5_class4","Int_engine_size5_class5","Int_engine_size6_class1","Int_engine_size6_class2","Int_engine_size6_class3","Int_engine_size6_class4","Int_engine_size6_class5","Int_engine_size7_class1","Int_engine_size7_class2","Int_engine_size7_class3","Int_engine_size8_class1","Int_engine_size8_class2","Int_engine_size8_class5","Int_fighterbay_size5_class1","Int_fighterbay_size6_class1","Int_fighterbay_size7_class1","Int_fsdinterdictor_size1_class1","Int_fsdinterdictor_size1_class2","Int_fsdinterdictor_size1_class3","Int_fsdinterdictor_size1_class4","Int_fsdinterdictor_size1_class5","Int_fsdinterdictor_size2_class1","Int_fsdinterdictor_size2_class2","Int_fsdinterdictor_size2_class3","Int_fsdinterdictor_size2_class4","Int_fsdinterdictor_size2_class5","Int_fsdinterdictor_size3_class1","Int_fsdinterdictor_size3_class2","Int_fsdinterdictor_size3_class3","Int_fsdinterdictor_size3_class4","Int_fsdinterdictor_size4_class1","Int_fsdinterdictor_size4_class2","Int_fsdinterdictor_size4_class3","Int_fsdinterdictor_size4_class4","Int_fsdinterdictor_size4_class5","Int_fuelscoop_size1_class1","Int_fuelscoop_size1_class2","Int_fuelscoop_size1_class3","Int_fuelscoop_size1_class4","Int_fuelscoop_size2_class1","Int_fuelscoop_size2_class2","Int_fuelscoop_size2_class3","Int_fuelscoop_size2_class4","Int_fuelscoop_size2_class5","Int_fuelscoop_size3_class1","Int_fuelscoop_size3_class2","Int_fuelscoop_size3_class3","Int_fuelscoop_size7_class1","Int_fuelscoop_size7_class2","Int_fuelscoop_size7_class4","Int_fuelscoop_size8_class1","Int_fuelscoop_size8_class2","Int_fuelscoop_size8_class3","Int_fuelscoop_size8_class4","Int_fueltank_size1_class3","Int_fueltank_size2_class3","Int_fueltank_size3_class3","Int_fueltank_size4_class3","Int_fueltank_size5_class3","Int_fueltank_size6_class3","Int_fueltank_size7_class3","Int_fueltank_size8_class3","Int_hullreinforcement_size1_class1","Int_hullreinforcement_size1_class2","Int_hullreinforcement_size2_class1","Int_hullreinforcement_size2_class2","Int_hullreinforcement_size3_class1","Int_hullreinforcement_size3_class2","Int_hullreinforcement_size4_class1","Int_hullreinforcement_size4_class2","Int_hullreinforcement_size5_class1","Int_hullreinforcement_size5_class2","Int_hyperdrive_overcharge_size2_class1","Int_hyperdrive_overcharge_size2_class2","Int_hyperdrive_overcharge_size2_class3","Int_hyperdrive_overcharge_size2_class4","Int_hyperdrive_overcharge_size2_class5","Int_hyperdrive_overcharge_size3_class1","Int_hyperdrive_overcharge_size3_class2","Int_hyperdrive_overcharge_size3_class3","Int_hyperdrive_overcharge_size3_class4","Int_hyperdrive_overcharge_size3_class5","Int_hyperdrive_overcharge_size4_class1","Int_hyperdrive_overcharge_size4_class2","Int_hyperdrive_overcharge_size4_class3","Int_hyperdrive_overcharge_size4_class4","Int_hyperdrive_overcharge_size4_class5","Int_hyperdrive_overcharge_size5_class1","Int_hyperdrive_overcharge_size5_class2","Int_hyperdrive_overcharge_size5_class3","Int_hyperdrive_overcharge_size5_class4","Int_hyperdrive_overcharge_size5_class5","Int_hyperdrive_overcharge_size6_class1","Int_hyperdrive_overcharge_size6_class2","Int_hyperdrive_overcharge_size6_class3","Int_hyperdrive_overcharge_size6_class4","Int_hyperdrive_overcharge_size6_class5","Int_hyperdrive_overcharge_size7_class1","Int_hyperdrive_overcharge_size7_class2","Int_hyperdrive_overcharge_size7_class3","Int_hyperdrive_overcharge_size7_class4","Int_hyperdrive_overcharge_size7_class5","Int_hyperdrive_size2_class1","Int_hyperdrive_size2_class2","Int_hyperdrive_size2_class3","Int_hyperdrive_size2_class4","Int_hyperdrive_size2_class5","Int_hyperdrive_size3_class1","Int_hyperdrive_size3_class2","Int_hyperdrive_size3_class3","Int_hyperdrive_size3_class4","Int_hyperdrive_size3_class5","Int_hyperdrive_size4_class1","Int_hyperdrive_size4_class2","Int_hyperdrive_size4_class3","Int_hyperdrive_size4_class4","Int_hyperdrive_size4_class5","Int_hyperdrive_size5_class1","Int_hyperdrive_size5_class2","Int_hyperdrive_size5_class3","Int_hyperdrive_size5_class4","Int_hyperdrive_size5_class5","Int_hyperdrive_size6_class1","Int_hyperdrive_size6_class2","Int_hyperdrive_size6_class3","Int_hyperdrive_size6_class4","Int_hyperdrive_size7_class1","Int_hyperdrive_size7_class3","Int_largecargorack_size7_class1","Int_largecargorack_size8_class1","Int_lifesupport_size1_class1","Int_lifesupport_size1_class2","Int_lifesupport_size1_class3","Int_lifesupport_size1_class4","Int_lifesupport_size1_class5","Int_lifesupport_size2_class1","Int_lifesupport_size2_class2","Int_lifesupport_size2_class3","Int_lifesupport_size2_class4","Int_lifesupport_size2_class5","Int_lifesupport_size3_class1","Int_lifesupport_size3_class2","Int_lifesupport_size3_class3","Int_lifesupport_size3_class4","Int_lifesupport_size4_class1","Int_lifesupport_size4_class2","Int_lifesupport_size4_class3","Int_lifesupport_size4_class4","Int_lifesupport_size4_class5","Int_lifesupport_size5_class1","Int_lifesupport_size5_class2","Int_lifesupport_size5_class3","Int_lifesupport_size5_class4","Int_lifesupport_size5_class5","Int_lifesupport_size6_class1","Int_lifesupport_size6_class2","Int_lifesupport_size6_class3","Int_lifesupport_size6_class4","Int_lifesupport_size6_class5","Int_lifesupport_size7_class1","Int_lifesupport_size7_class2","Int_lifesupport_size7_class3","Int_lifesupport_size7_class4","Int_lifesupport_size7_class5","Int_lifesupport_size8_class1","Int_lifesupport_size8_class4","Int_lifesupport_size8_class5","Int_modulereinforcement_size1_class1","Int_modulereinforcement_size1_class2","Int_modulereinforcement_size2_class1","Int_modulereinforcement_size3_class1","Int_modulereinforcement_size3_class2","Int_modulereinforcement_size4_class1","Int_modulereinforcement_size4_class2","Int_modulereinforcement_size5_class1","Int_modulereinforcement_size5_class2","Int_multidronecontrol_mining_size3_class1","Int_multidronecontrol_mining_size3_class3","Int_multidronecontrol_operations_size3_class3","Int_multidronecontrol_operations_size3_class4","Int_multidronecontrol_rescue_size3_class2","Int_multidronecontrol_rescue_size3_class3","Int_multidronecontrol_xeno_size3_class3","Int_multidronecontrol_xeno_size3_class4","Int_passengercabin_size2_class1","Int_passengercabin_size3_class2","Int_passengercabin_size4_class1","Int_passengercabin_size4_class2","Int_passengercabin_size4_class3","Int_passengercabin_size5_class1","Int_passengercabin_size5_class2","Int_passengercabin_size5_class3","Int_passengercabin_size6_class1","Int_passengercabin_size6_class2","Int_passengercabin_size6_class3","Int_passengercabin_size6_class4","Int_powerdistributor_size1_class1","Int_powerdistributor_size1_class2","Int_powerdistributor_size1_class3","Int_powerdistributor_size1_class5","Int_powerdistributor_size2_class1","Int_powerdistributor_size2_class2","Int_powerdistributor_size2_class3","Int_powerdistributor_size2_class4","Int_powerdistributor_size2_class5","Int_powerdistributor_size3_class1","Int_powerdistributor_size3_class2","Int_powerdistributor_size3_class3","Int_powerdistributor_size3_class4","Int_powerdistributor_size3_class5","Int_powerdistributor_size4_class1","Int_powerdistributor_size4_class2","Int_powerdistributor_size4_class3","Int_powerdistributor_size4_class5","Int_powerdistributor_size5_class1","Int_powerdistributor_size5_class2","Int_powerdistributor_size5_class3","Int_powerdistributor_size5_class4","Int_powerdistributor_size5_class5","Int_powerdistributor_size6_class1","Int_powerdistributor_size6_class2","Int_powerdistributor_size6_class3","Int_powerdistributor_size6_class4","Int_powerdistributor_size7_class1","Int_powerdistributor_size7_class2","Int_powerdistributor_size7_class3","Int_powerdistributor_size7_class4","Int_powerdistributor_size7_class5","Int_powerdistributor_size8_class1","Int_powerdistributor_size8_class2","Int_powerdistributor_size8_class4","Int_powerdistributor_size8_class5","Int_powerplant_size2_class1","Int_powerplant_size2_class2","Int_powerplant_size2_class3","Int_powerplant_size2_class4","Int_powerplant_size2_class5","Int_powerplant_size3_class1","Int_powerplant_size3_class2","Int_powerplant_size3_class3","Int_powerplant_size3_class4","Int_powerplant_size3_class5","Int_powerplant_size4_class1","Int_powerplant_size4_class2","Int_powerplant_size4_class3","Int_powerplant_size4_class4","Int_powerplant_size4_class5","Int_powerplant_size5_class1","Int_powerplant_size5_class2","Int_powerplant_size5_class3","Int_powerplant_size5_class4","Int_powerplant_size5_class5","Int_powerplant_size6_class1","Int_powerplant_size6_class2","Int_powerplant_size6_class3","Int_powerplant_size6_class4","Int_powerplant_size7_class1","Int_powerplant_size7_class2","Int_powerplant_size7_class3","Int_powerplant_size7_class5","Int_powerplant_size8_class1","Int_powerplant_size8_class2","Int_powerplant_size8_class3","Int_refinery_size1_class1","Int_refinery_size1_class2","Int_refinery_size1_class3","Int_refinery_size1_class4","Int_refinery_size1_class5","Int_refinery_size2_class1","Int_refinery_size2_class2","Int_refinery_size2_class3","Int_refinery_size2_class4","Int_refinery_size2_class5","Int_refinery_size3_class1","Int_refinery_size3_class2","Int_refinery_size3_class3","Int_refinery_size3_class4","Int_refinery_size3_class5","Int_refinery_size4_class1","Int_refinery_size4_class2","Int_refinery_size4_class3","Int_refinery_size4_class4","Int_repairer_size1_class1","Int_repairer_size1_class2","Int_repairer_size1_class3","Int_repairer_size1_class4","Int_repairer_size1_class5","Int_repairer_size2_class1","Int_repairer_size2_class2","Int_repairer_size2_class3","Int_repairer_size2_class4","Int_repairer_size2_class5","Int_repairer_size3_class1","Int_repairer_size3_class2","Int_repairer_size3_class3","Int_repairer_size3_class4","Int_repairer_size3_class5","Int_repairer_size4_class1","Int_repairer_size4_class2","Int_repairer_size4_class3","Int_repairer_size4_class4","Int_repairer_size4_class5","Int_repairer_size5_class1","Int_repairer_size5_class2","Int_repairer_size5_class3","Int_repairer_size5_class4","Int_repairer_size6_class1","Int_repairer_size6_class2","Int_repairer_size6_class3","Int_repairer_size6_class4","Int_repairer_size7_class1","Int_repairer_size7_class2","Int_repairer_size7_class3","Int_repairer_size8_class1","Int_repairer_size8_class2","Int_repairer_size8_class3","Int_repairer_size8_class4","Int_sensors_size1_class1","Int_sensors_size1_class2","Int_sensors_size1_class3","Int_sensors_size1_class4","Int_sensors_size1_class5","Int_sensors_size2_class1","Int_sensors_size2_class2","Int_sensors_size2_class3","Int_sensors_size2_class4","Int_sensors_size2_class5","Int_sensors_size3_class1","Int_sensors_size3_class2","Int_sensors_size3_class3","Int_sensors_size3_class4","Int_sensors_size3_class5","Int_sensors_size4_class1","Int_sensors_size4_class2","Int_sensors_size4_class3","Int_sensors_size4_class4","Int_sensors_size4_class5","Int_sensors_size5_class1","Int_sensors_size5_class2","Int_sensors_size5_class3","Int_sensors_size5_class4","Int_sensors_size6_class1","Int_sensors_size6_class2","Int_sensors_size6_class3","Int_sensors_size6_class4","Int_sensors_size7_class1","Int_sensors_size7_class2","Int_sensors_size7_class3","Int_sensors_size7_class4","Int_sensors_size7_class5","Int_sensors_size8_class1","Int_sensors_size8_class2","Int_sensors_size8_class3","Int_sensors_size8_class5","Int_shieldcellbank_size1_class1","Int_shieldcellbank_size1_class2","Int_shieldcellbank_size1_class3","Int_shieldcellbank_size1_class4","Int_shieldcellbank_size1_class5","Int_shieldcellbank_size2_class1","Int_shieldcellbank_size2_class2","Int_shieldcellbank_size2_class3","Int_shieldcellbank_size2_class5","Int_shieldcellbank_size3_class1","Int_shieldcellbank_size3_class2","Int_shieldcellbank_size3_class3","Int_shieldcellbank_size3_class4","Int_shieldcellbank_size3_class5","Int_shieldcellbank_size4_class1","Int_shieldcellbank_size4_class2","Int_shieldcellbank_size4_class3","Int_shieldcellbank_size4_class4","Int_shieldcellbank_size4_class5","Int_shieldcellbank_size5_class1","Int_shieldcellbank_size5_class2","Int_shieldcellbank_size5_class4","Int_shieldcellbank_size6_class1","Int_shieldcellbank_size6_class2","Int_shieldcellbank_size6_class3","Int_shieldcellbank_size6_class4","Int_shieldcellbank_size6_class5","Int_shieldcellbank_size7_class1","Int_shieldcellbank_size7_class2","Int_shieldcellbank_size7_class3","Int_shieldcellbank_size7_class4","Int_shieldcellbank_size7_class5","Int_shieldcellbank_size8_class1","Int_shieldcellbank_size8_class3","Int_shieldcellbank_size8_class4","Int_shieldgenerator_size1_class3_fast","Int_shieldgenerator_size2_class1","Int_shieldgenerator_size2_class2","Int_shieldgenerator_size2_class3","Int_shieldgenerator_size2_class3_fast","Int_shieldgenerator_size2_class4","Int_shieldgenerator_size2_class5","Int_shieldgenerator_size3_class1","Int_shieldgenerator_size3_class2","Int_shieldgenerator_size3_class3","Int_shieldgenerator_size3_class3_fast","Int_shieldgenerator_size3_class4","Int_shieldgenerator_size3_class5","Int_shieldgenerator_size4_class1","Int_shieldgenerator_size4_class2","Int_shieldgenerator_size4_class3","Int_shieldgenerator_size4_class3_fast","Int_shieldgenerator_size4_class4","Int_shieldgenerator_size4_class5","Int_shieldgenerator_size5_class1","Int_shieldgenerator_size5_class2","Int_shieldgenerator_size5_class3","Int_shieldgenerator_size5_class3_fast","Int_shieldgenerator_size5_class4","Int_shieldgenerator_size5_class5","Int_shieldgenerator_size6_class1","Int_shieldgenerator_size6_class2","Int_shieldgenerator_size6_class3","Int_shieldgenerator_size6_class3_fast","Int_shieldgenerator_size6_class4","Int_shieldgenerator_size6_class5","Int_shieldgenerator_size7_class1","Int_shieldgenerator_size7_class2","Int_shieldgenerator_size7_class3_fast","Int_shieldgenerator_size7_class4","Int_shieldgenerator_size7_class5","Int_shieldgenerator_size8_class1","Int_shieldgenerator_size8_class2","Int_shieldgenerator_size8_class3","Int_shieldgenerator_size8_class3_fast","Int_shieldgenerator_size8_class4","Int_shieldgenerator_size8_class5","Int_supercruiseassist","anaconda_Armour_grade1","anaconda_Armour_grade2","anaconda_Armour_grade3","anaconda_Armour_mirrored","anaconda_Armour_reactive","asp_Armour_grade1","asp_Armour_grade2","asp_Armour_grade3","asp_Armour_mirrored","asp_Armour_reactive","asp_scout_Armour_grade1","asp_scout_Armour_grade2","asp_scout_Armour_grade3","asp_scout_Armour_mirrored","asp_scout_Armour_reactive","belugaliner_Armour_grade1","belugaliner_Armour_grade2","belugaliner_Armour_grade3","belugaliner_Armour_mirrored","belugaliner_Armour_reactive","cobramkiii_Armour_grade1","cobramkiii_Armour_grade2","cobramkiii_Armour_grade3","cobramkiii_Armour_mirrored","cobramkiii_Armour_reactive","cobramkiv_Armour_grade1","cobramkiv_Armour_grade2","cobramkiv_Armour_grade3","cobramkiv_Armour_mirrored","cobramkiv_Armour_reactive","diamondback_Armour_grade1","diamondback_Armour_grade2","diamondback_Armour_grade3","diamondback_Armour_mirrored","diamondback_Armour_reactive","diamondbackxl_Armour_grade1","diamondbackxl_Armour_grade2","diamondbackxl_Armour_grade3","diamondbackxl_Armour_mirrored","diamondbackxl_Armour_reactive","dolphin_Armour_grade1","dolphin_Armour_grade2","dolphin_Armour_grade3","dolphin_Armour_mirrored","dolphin_Armour_reactive","eagle_Armour_grade1","eagle_Armour_grade2","eagle_Armour_grade3","eagle_Armour_mirrored","eagle_Armour_reactive","ferdelance_Armour_grade1","ferdelance_Armour_grade2","ferdelance_Armour_grade3","ferdelance_Armour_mirrored","ferdelance_Armour_reactive","hauler_Armour_grade1","hauler_Armour_grade2","hauler_Armour_grade3","hauler_Armour_mirrored","hauler_Armour_reactive","independant_trader_Armour_grade1","independant_trader_Armour_grade2","independant_trader_Armour_grade3","independant_trader_Armour_mirrored","independant_trader_Armour_reactive","krait_light_Armour_grade1","krait_light_Armour_grade2","krait_light_Armour_grade3","krait_light_Armour_mirrored","krait_light_Armour_reactive","krait_mkii_Armour_grade1","krait_mkii_Armour_grade2","krait_mkii_Armour_grade3","krait_mkii_Armour_mirrored","krait_mkii_Armour_reactive","mamba_Armour_grade1","mamba_Armour_grade2","mamba_Armour_grade3","mamba_Armour_mirrored","mamba_Armour_reactive","python_Armour_grade1","python_Armour_grade2","python_Armour_grade3","python_Armour_mirrored","python_Armour_reactive","sidewinder_Armour_grade1","sidewinder_Armour_grade2","sidewinder_Armour_grade3","sidewinder_Armour_mirrored","sidewinder_Armour_reactive","type6_Armour_grade1","type6_Armour_grade2","type6_Armour_grade3","type6_Armour_mirrored","type6_Armour_reactive","type7_Armour_grade1","type7_Armour_grade2","type7_Armour_grade3","type7_Armour_mirrored","type7_Armour_reactive","type9_Armour_grade1","type9_Armour_grade2","type9_Armour_grade3","type9_Armour_mirrored","type9_Armour_reactive","type9_military_Armour_grade1","type9_military_Armour_grade2","typex_2_Armour_grade1","typex_2_Armour_grade2","typex_2_Armour_grade3","typex_3_Armour_grade1","typex_3_Armour_grade2","typex_3_Armour_grade3","viper_Armour_grade1","viper_Armour_grade2","viper_Armour_grade3","viper_Armour_mirrored","viper_Armour_reactive","viper_mkiv_Armour_grade1","viper_mkiv_Armour_grade2","viper_mkiv_Armour_grade3","viper_mkiv_Armour_mirrored","viper_mkiv_Armour_reactive","vulture_Armour_grade1","vulture_Armour_grade2","vulture_Armour_grade3","vulture_Armour_mirrored","vulture_Armour_reactive"],"odyssey":true,"stationName":"Soundand Vision","systemName":"Delta-2 Chamaelontis","timestamp":"2025-09-08T14:50:10Z"}
            error!(
                "Not implemented: MARKET_ID AND NO COMMODITIES AND NO SHIPS: {}",
                journal_id
            );
        }

        "unknown" => {
            error!("UNKNOWN JOURNAL: {}", json);
        }

        "Fileheader" => {
            return Ok(());
        }
        "Shutdown" => {
            return Ok(());
        }
        "" => {
            return Ok(());
        }
        _ => {
            error!("Unknown event: {}", &journal_id);
        }
    }
    Err(EdcasError(format!("[{}] No event interpreted", journal_id)))
}
