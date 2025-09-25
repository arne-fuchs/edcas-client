use serde::{Deserialize, Serialize};

use crate::edcas::assets::station::{LandingPads, StationEconomy, StationFaction};

#[derive(Serialize, Deserialize)]
pub struct Docked {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "StationFaction")]
    station_faction: StationFaction,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "StationAllegiance")]
    station_allegiance: Option<String>,

    #[serde(rename = "MarketID")]
    market_id: i64,

    #[serde(rename = "DistFromStarLS")]
    dist_from_star_ls: f32,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "StationGovernment")]
    station_government: String,

    #[serde(rename = "odyssey")]
    #[serde(default)]
    odyssey: bool,

    #[serde(rename = "horizons")]
    #[serde(default)]
    horizons: bool,

    #[serde(rename = "StationName")]
    station_name: String,

    #[serde(rename = "StationServices")]
    station_services: Vec<String>,

    #[serde(rename = "Multicrew")]
    #[serde(default)]
    multicrew: bool,

    #[serde(rename = "StationEconomy")]
    station_economy: String,

    #[serde(rename = "Taxi")]
    #[serde(default)]
    taxi: bool,

    #[serde(rename = "StationEconomies")]
    station_economies: Vec<StationEconomy>,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "LandingPads")]
    landing_pads: Option<LandingPads>,

    #[serde(rename = "timestamp")]
    timestamp: String,

    #[serde(rename = "StationType")]
    station_type: String,
}
impl Docked {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};

        let Self {
            system_address,
            station_faction,
            star_pos,
            station_allegiance,
            market_id,
            dist_from_star_ls,
            star_system,
            station_government,
            odyssey: _,
            horizons: _,
            station_name,
            station_services,
            multicrew: _,
            station_economy,
            taxi: _,
            station_economies,
            event: _,
            landing_pads,
            timestamp,
            station_type,
        } = self;
        //TODO Does this even work?
        if let Err(err) = client.execute(
            //language_postgres
            "INSERT INTO star_systems (system_address,name,x,y,z,journal_id) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
            &[&system_address,&star_system,&star_pos[0],&star_pos[1],&star_pos[2],&journal_id]
        ){
            return Err(EdcasError::new(format!("Star: {}", err)));
        }
        let mut transaction = client.transaction()?;
        let station_type = (
            value_table(
                Tables::StationType,
                station_type.clone(),
                journal_id,
                &mut transaction,
            )?,
            station_type,
        );

        let station_allegiance = value_table(
            Tables::Allegiance,
            match station_allegiance {
                Some(f) => f,
                None => "".to_string(),
            },
            journal_id,
             &mut transaction,
        )?;

        let station_government =
            value_table(Tables::Government, station_government, journal_id,  &mut transaction)?;
        let station_economy =
            value_table(Tables::EconomyType, station_economy, journal_id,  &mut transaction)?;

        //TODO delete old station economy
        for station_economy in station_economies {
            station_economy.insert_into_db(journal_id,  &mut transaction)?;
        }

        let station_faction = station_faction.insert_into_db(journal_id,  &mut transaction)?;
        let happiness = value_table(Tables::Happiness, "".to_string(), journal_id,  &mut transaction)?;
        use crate::eddn::edcas_error::EdcasError;
        match station_type.1.as_str() {
            "FleetCarrier" | "PlanetaryConstructionDepot" | "SpaceConstructionDepot" => {
                if let Err(err) = transaction.execute(
                                                //language=postgresql
                                                "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                                      government=$3,allegiance=$4,journal_id=$7",
                                                &[&station_faction,&system_address,&station_government,&station_allegiance,&happiness,&0.0f32,&journal_id]
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
                                                &[&station_faction,&system_address,&station_government,&station_allegiance,&happiness,&0.0f32,&journal_id]
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
                                            &[&station_faction,&system_address,&station_government,&station_allegiance,&happiness,&0.0f32,&journal_id]
                                        ) {
                    log::error!("[Megaship]insert station faction: {}",err);
                    return Err(EdcasError::from(err));
                }
            }
        }
        transaction.commit()?;
        //Market
        if let Err(err) = client.execute(
            //language=postgres
            "INSERT INTO stations (market_id, system_address, name, type, faction_name, government, economy, journal_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) on conflict on constraint stations_pkey do update SET
                                            system_address=$2,
                                            name=$3,
                                            type=$4,
                                            faction_name=$5,
                                            government=$6,
                                            economy=$7,
                                            journal_id=$8",
                                            &[&market_id,&system_address,&station_name,&station_type.0,&station_faction,&station_allegiance,&station_economy,&journal_id]
            ){
                log::error!("[Stations]insert station: {}",err);
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
        if let Some(landing_pads) = landing_pads{
            landing_pads.insert_into_db(journal_id, market_id, &mut transaction)?;
        }
        transaction.commit()?;
        Ok(())
    }
}
