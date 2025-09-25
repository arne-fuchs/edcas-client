use serde::{Deserialize, Serialize};

use crate::edcas::assets::station::{StationEconomy, StationFaction};

#[derive(Serialize, Deserialize)]
pub struct Carrierjump {
    #[serde(rename = "StationFaction")]
    station_faction: StationFaction,

    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "SystemSecondEconomy")]
    system_second_economy: String,

    #[serde(rename = "BodyType")]
    body_type: String,

    #[serde(rename = "SystemGovernment")]
    system_government: String,

    #[serde(rename = "SystemAllegiance")]
    system_allegiance: String,

    #[serde(rename = "SystemEconomy")]
    system_economy: String,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "StationName")]
    station_name: String,

    #[serde(rename = "StationEconomy")]
    station_economy: String,

    #[serde(rename = "Population")]
    population: i64,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "Body")]
    body: String,

    #[serde(rename = "StationType")]
    station_type: String,

    #[serde(rename = "timestamp")]
    timestamp: String,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "SystemSecurity")]
    system_security: String,

    #[serde(rename = "MarketID")]
    market_id: i64,

    #[serde(rename = "Docked")]
    docked: bool,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "StationGovernment")]
    station_government: String,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "StationServices")]
    station_services: Vec<String>,

    #[serde(rename = "BodyID")]
    body_id: i64,

    #[serde(rename = "StationEconomies")]
    station_economies: Vec<StationEconomy>,

    #[serde(rename = "Powers")]
    powers: Option<Vec<String>>,

    #[serde(rename = "ControllingPower")]
    controlling_power: Option<String>,
}
impl Carrierjump {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            station_faction,
            system_address,
            system_second_economy,
            body_type,
            system_government,
            system_allegiance,
            system_economy,
            odyssey: _,
            station_name,
            station_economy,
            population,
            event: _,
            body,
            station_type,
            timestamp,
            star_pos,
            system_security,
            market_id,
            docked: _,
            star_system,
            station_government,
            horizons: _,
            station_services,
            body_id,
            station_economies,
            powers,
            controlling_power,
        } = self;
        //TODO Implement
        let _ = body_id;
        let _ = timestamp;
        let _ = body_type;
        let _ = body;
        //TODO Remove old carrier status

        //TODO Missing: Thargoid war status. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Power play status is mssing. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Check if actually something is being saved
        if let Some(powers) = powers {
            for power in powers {
                value_table(Tables::Power, power, journal_id, client)?;
            }
        }
        let controlling_power = if let Some(controlling_power) = controlling_power {
            Some(value_table(
                Tables::Power,
                controlling_power,
                journal_id,
                client,
            )?)
        } else {
            None
        };
        let station_faction = station_faction.insert_into_db(journal_id, client)?;
        let system_government =
            value_table(Tables::Government, system_government, journal_id, client)?;
        let station_government =
            value_table(Tables::Government, station_government, journal_id, client)?;
        let station_type = value_table(Tables::StationType, station_type, journal_id, client)?;

        let system_allegiance =
            value_table(Tables::Allegiance, system_allegiance, journal_id, client)?;
        let economy = value_table(Tables::EconomyType, system_economy, journal_id, client)?;
        let second_economy = value_table(
            Tables::EconomyType,
            system_second_economy,
            journal_id,
            client,
        )?;
        let security = value_table(Tables::Security, system_security, journal_id, client)?;
        let system_address = crate::edcas::assets::star_system::insert_star_system(
            system_address,
            star_system,
            (star_pos[0], star_pos[1], star_pos[2]),
            system_allegiance,
            economy,
            second_economy,
            system_government,
            security,
            population,
            controlling_power,
            journal_id,
            client,
        )?;

        let allegiance = value_table(
            Tables::Allegiance,
            "PilotsFederation".to_string(),
            journal_id,
            client,
        )?;
        let happiness = value_table(Tables::Happiness, "".to_string(), journal_id, client)?;
        if let Err(err) = client.execute(
                                        //language=postgresql
                                        "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                              government=$3,allegiance=$4,journal_id=$7",
                                        &[&station_faction,&system_address,&system_government,&allegiance,&happiness,&0.0f32,&journal_id]
                                    ) {
                                        use crate::eddn::edcas_error::EdcasError;

            log::error!("[{}]insert ConstructionDepot faction: {}",journal_id,err);
            return Err(EdcasError::from(err));
        }
        //Market
        let station_economy =
            value_table(Tables::EconomyType, station_economy, journal_id, client)?;
        for station_economy in station_economies {
            station_economy.insert_into_db(journal_id, client)?;
        }
        if let Err(err) = client.execute(
            //language=postgres
            "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) ON CONFLICT ON CONSTRAINT stations_pkey DO UPDATE SET
                                            system_address=$2,
                                            name=$3,
                                            type=$4,
                                            faction_name=$5,
                                            government=$6,
                                            economy=$7,
                                            journal_id=$8",
                                            &[&market_id,&system_address,&station_name,&station_type,&station_faction,&station_government,&station_economy,&journal_id]
            ){
                log::error!("[{}]insert station: {}",journal_id,err);
                return Err(EdcasError::from(err));
            }

        //Station services
        if let Err(err) = client.execute(
            "DELETE FROM station_services WHERE market_id=$1",
            &[&market_id],
        ) {
            log::error!("[{}]delete station services: {}", journal_id, err);
            return Err(EdcasError::from(err));
        }
        for station_service in station_services {
            let id = value_table(
                Tables::StationServicesTypes,
                station_service,
                journal_id,
                client,
            )?;
            if let Err(err) = client.execute(
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
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Carrierjumponfoot {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "SystemSecurity")]
    system_security: String,

    #[serde(rename = "Docked")]
    docked: bool,

    #[serde(rename = "SystemSecondEconomy")]
    system_second_economy: String,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "BodyType")]
    body_type: String,

    #[serde(rename = "SystemGovernment")]
    system_government: String,

    #[serde(rename = "SystemEconomy")]
    system_economy: String,

    #[serde(rename = "SystemAllegiance")]
    system_allegiance: String,

    #[serde(rename = "odyssey")]
    #[serde(default)]
    odyssey: bool,

    #[serde(rename = "horizons")]
    #[serde(default)]
    horizons: bool,

    #[serde(rename = "OnFoot")]
    on_foot: bool,

    #[serde(rename = "BodyID")]
    body_id: i64,

    #[serde(rename = "Population")]
    population: i64,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "Body")]
    body: String,

    #[serde(rename = "timestamp")]
    timestamp: String,

    #[serde(rename = "Powers")]
    powers: Option<Vec<String>>,

    #[serde(rename = "ControllingPower")]
    controlling_power: Option<String>,
}
impl Carrierjumponfoot {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        let Self {
            system_address,
            star_pos,
            system_security,
            docked: _,
            system_second_economy,
            star_system,
            body_type,
            system_government,
            system_economy,
            system_allegiance,
            odyssey: _,
            horizons: _,
            on_foot: _,
            body_id,
            population,
            event: _,
            body,
            timestamp,
            powers,
            controlling_power,
        } = self;
        //TODO: Implement
        let _ = body_id;
        let _ = timestamp;
        let _ = body_type;
        let _ = body;

        //TODO Remove old carrier status

        //TODO Missing: Thargoid war status. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Power play status is mssing. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Check if actually something is being saved
        use crate::edcas::tables::{value_table, Tables};
        if let Some(powers) = powers {
            for power in powers {
                value_table(Tables::Power, power, journal_id, client)?;
            }
        }
        let controlling_power = if let Some(controlling_power) = controlling_power {
            Some(value_table(
                Tables::Power,
                controlling_power,
                journal_id,
                client,
            )?)
        } else {
            None
        };
        let system_government =
            value_table(Tables::Government, system_government, journal_id, client)?;
        let system_allegiance =
            value_table(Tables::Allegiance, system_allegiance, journal_id, client)?;
        let economy = value_table(Tables::EconomyType, system_economy, journal_id, client)?;
        let second_economy = value_table(
            Tables::EconomyType,
            system_second_economy,
            journal_id,
            client,
        )?;
        let security = value_table(Tables::Security, system_security, journal_id, client)?;
        crate::edcas::assets::star_system::insert_star_system(
            system_address,
            star_system,
            (star_pos[0], star_pos[1], star_pos[2]),
            system_allegiance,
            economy,
            second_economy,
            system_government,
            security,
            population,
            controlling_power,
            journal_id,
            client,
        )?;
        Ok(())
    }
}
