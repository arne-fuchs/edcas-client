use serde::{Deserialize, Serialize};

use crate::edcas::assets::{
    faction::{conflict::Conflict, Faction, SystemFaction},
    station::{StationEconomy, StationFaction},
};

#[derive(Serialize, Deserialize)]
pub struct Location {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "SystemSecondEconomy")]
    system_second_economy: String,

    #[serde(rename = "DistFromStarLS")]
    dist_from_star_ls: Option<f32>,

    #[serde(rename = "BodyType")]
    body_type: String,

    #[serde(rename = "SystemGovernment")]
    system_government: String,

    #[serde(rename = "SystemAllegiance")]
    system_allegiance: String,

    #[serde(rename = "SystemEconomy")]
    system_economy: String,

    #[serde(rename = "odyssey")]
    #[serde(default)]
    odyssey: bool,

    #[serde(rename = "Population")]
    population: i64,

    #[serde(rename = "Taxi")]
    #[serde(default)]
    taxi: bool,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "Body")]
    body: String,

    #[serde(rename = "timestamp")]
    timestamp: String,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "SystemSecurity")]
    system_security: String,

    #[serde(rename = "Factions")]
    factions: Option<Vec<Faction>>,

    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<Conflict>>,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "horizons")]
    #[serde(default)]
    horizons: bool,

    #[serde(rename = "Multicrew")]
    #[serde(default)]
    multicrew: bool,

    #[serde(rename = "SystemFaction")]
    system_faction: Option<SystemFaction>,

    #[serde(rename = "BodyID")]
    body_id: i64,

    #[serde(rename = "Powers")]
    powers: Option<Vec<String>>,

    #[serde(rename = "ControllingPower")]
    controlling_power: Option<String>,

    #[serde(rename = "Docked")]
    docked: bool,
    //When docked, these variables are available
    #[serde(rename = "StationName")]
    station_name: Option<String>,

    #[serde(rename = "MarketID")]
    market_id: Option<i64>,

    #[serde(rename = "StationEconomy")]
    station_economy: Option<String>,

    #[serde(rename = "StationType")]
    station_type: Option<String>,

    #[serde(rename = "StationGovernment")]
    station_government: Option<String>,

    #[serde(rename = "StationEconomies")]
    station_economies: Option<Vec<StationEconomy>>,

    #[serde(rename = "StationFaction")]
    station_faction: Option<StationFaction>,

    #[serde(rename = "StationServices")]
    station_services: Option<Vec<String>>,
}
impl Location {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};

        let Self {
            system_address,
            system_second_economy,
            dist_from_star_ls,
            body_type,
            system_government,
            system_allegiance,
            system_economy,
            odyssey: _,
            population,
            taxi: _,
            event: _,
            body,
            timestamp,
            star_pos,
            system_security,
            factions,
            conflicts,
            star_system,
            horizons: _,
            station_services,
            multicrew: _,
            system_faction,
            body_id,
            powers,
            controlling_power,
            docked,
            station_name,
            market_id,
            station_economy,
            station_type,
            station_government,
            station_economies,
            station_faction,
        } = self;
        let mut transaction = client.transaction()?;
        if let Some(system_faction) = system_faction {
            system_faction.insert_into_db(journal_id, &mut transaction)?;
        }
        //TODO Implement
        let _ = dist_from_star_ls;
        let _ = body_id;
        let _ = body;
        let _ = body_type;
        let _ = timestamp;
        //TODO Missing: Thargoid war status. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Power play status is mssing. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Check if actually something is being saved
        if let Some(powers) = powers {
            for power in powers {
                value_table(Tables::Power, power, journal_id, &mut transaction)?;
            }
        }
        let controlling_power = if let Some(controlling_power) = controlling_power {
            Some(value_table(
                Tables::Power,
                controlling_power,
                journal_id,
                &mut transaction,
            )?)
        } else {
            None
        };
        let system_allegiance =
            value_table(Tables::Allegiance, system_allegiance, journal_id, &mut transaction)?;
        let economy = value_table(Tables::EconomyType, system_economy, journal_id, &mut transaction)?;
        let second_economy = value_table(
            Tables::EconomyType,
            system_second_economy,
            journal_id,
            &mut transaction,
        )?;
        let government = value_table(Tables::Government, system_government, journal_id, &mut transaction)?;
        let security = value_table(Tables::Security, system_security, journal_id, &mut transaction)?;
        transaction.commit()?;

        let system_address = crate::edcas::assets::star_system::insert_star_system(
            system_address,
            star_system,
            (star_pos[0], star_pos[1], star_pos[2]),
            system_allegiance,
            economy,
            second_economy,
            government,
            security,
            population,
            controlling_power,
            journal_id,
            client,
        )?;

        let mut transaction = client.transaction()?;

        if let Some(factions) = factions {
            for faction in factions {
                faction.insert_into_db(journal_id, system_address, &mut transaction)?;
            }
        }
        if let Some(conflicts) = conflicts {
            for conflict in conflicts {
                conflict.insert_into_db(journal_id, system_address, &mut transaction)?;
            }
        }

        if docked {
            let market_id = market_id.ok_or(format!("No market id when docked: {}", journal_id))?;
            let station_name =
                station_name.ok_or(format!("No station name when docked: {}", journal_id))?;
            let station_type =
                station_type.ok_or(format!("No station type when docked: {}", journal_id))?;
            let station_type = (
                value_table(
                    Tables::StationType,
                    station_type.clone(),
                    journal_id,
                    &mut transaction,
                )?,
                station_type,
            );

            let station_government = station_government
                .ok_or(format!("No station government when docked: {}", journal_id))?;
            let _ = value_table(Tables::Government, station_government, journal_id, &mut transaction)?;

            let station_economy =
                station_economy.ok_or("No station economy when docked".to_string())?;

            let station_faction = station_faction
                .ok_or("No station faction when docked".to_string())?
                .insert_into_db(journal_id, &mut transaction)?;
            let happiness = value_table(Tables::Happiness, "".to_string(), journal_id, &mut transaction)?;
            use crate::eddn::edcas_error::EdcasError;
            match station_type.1.as_str() {
                "FleetCarrier" | "PlanetaryConstructionDepot" | "SpaceConstructionDepot" => {
                    let allegiance = value_table(
                        Tables::Allegiance,
                        "PilotsFederation".to_string(),
                        journal_id,
                        &mut transaction,
                    )?;
                    if let Err(err) = transaction.execute(
                                                    //language=postgresql
                                                    "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                                          government=$3,allegiance=$4,journal_id=$7",
                                                    &[&station_faction,&system_address,&government,&allegiance,&happiness,&0.0f32,&journal_id]
                                                ) {
                        log::error!("[{}]insert ConstructionDepot faction: {}",journal_id,err);
                        return Err(EdcasError::from(err));
                    }
                }
                "MegaShip" => {
                    if let Err(err) = transaction.execute(
                                                    //language=postgresql
                                                    "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                                          government=$3,allegiance=$4,journal_id=$7",
                                                    &[&station_faction,&system_address,&government,&system_allegiance,&happiness,&0.0f32,&journal_id]
                                                ) {

                        log::error!("[{}]insert megaship faction: {}",journal_id,err);
                        return Err(EdcasError::from(err));
                    }
                }
                _ => {
                    if let Err(err) = transaction.execute(
                                                //language=postgresql
                                                "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                                          government=$3,allegiance=$4,journal_id=$7",
                                                &[&station_faction,&system_address,&government,&system_allegiance,&happiness,&0.0f32,&journal_id]
                                            ) {
                        log::error!("[{}]insert station faction: {}",journal_id,err);
                        return Err(EdcasError::from(err));
                    }
                }
            }
            //TODO delete old station economy
            let _ = value_table(Tables::EconomyType, station_economy, journal_id, &mut transaction)?;
            if let Some(station_economies) = station_economies {
                for station_economy in station_economies {
                    station_economy.insert_into_db(journal_id, &mut transaction)?;
                }
            }
            transaction.commit()?;
            //Market
            if let Err(err) = client.execute(
                //language=postgres
                "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)on conflict on constraint stations_pkey do update SET
                                                system_address=$2,
                                                name=$3,
                                                type=$4,
                                                faction_name=$5,
                                                government=$6,
                                                economy=$7,
                                                journal_id=$8",
                                                &[&market_id,&system_address,&station_name,&station_type.0,&station_faction,&government,&economy,&journal_id]
                ){
                    log::error!("[Location]insert station: {}",err);
                    return Err(EdcasError::from(err));
                }
            let mut transaction = client.transaction()?;
            //Station services
            if let Err(err) = transaction.execute(
                "DELETE FROM station_services WHERE market_id=$1",
                &[&market_id],
            ) {
                log::error!("[{}]delete station services: {}", journal_id, err);
                return Err(EdcasError::from(err));
            }
            if let Some(station_services) = station_services {
                for station_service in station_services {
                    let id = value_table(
                        Tables::StationServicesTypes,
                        station_service,
                        journal_id,
                        &mut transaction,
                    )?;
                    if let Err(err) = transaction.execute(
                        // language=postgresql
                        "INSERT INTO station_services (id, market_id,journal_id) VALUES ($1, $2,$3)",
                        &[&id, &market_id, &journal_id],
                    ) {
                        log::error!(
                            "[{}]Insert station services: couldn't insert station service: {}",
                            journal_id,
                            err
                        );
                        return Err(EdcasError::from(err));
                    }
                }
            }
            transaction.commit()?;
        }
        Ok(())
    }
}
