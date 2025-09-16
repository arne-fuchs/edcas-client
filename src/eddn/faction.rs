use json::JsonValue;
use log::error;
use postgres::{Client, Error};

use crate::eddn::interpreter::{value_table, Tables};

pub fn insert_station_factions(
    client: &mut Client,
    json: &JsonValue,
    faction_name: i32,
    government: i32,
    system_allegiance: i32,
    system_address: &i64,
    journal_id: i64,
) -> Result<(), postgres::Error> {
    //Fleet carrier have faction "FleetCarrier" not present in the factions list of system
    let happiness = value_table(Tables::Happiness, "".to_string(), journal_id, client)?;

    if json["StationType"].to_string() == "FleetCarrier" {
        let allegiance = value_table(
            Tables::Allegiance,
            "PilotsFederation".to_string(),
            journal_id,
            client,
        )?;
        match client.execute(
            //language=postgresql
            "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                      government=$3,allegiance=$4,journal_id=$7",
            &[&faction_name,&system_address,&government,&allegiance,&happiness,&0.0f32,&journal_id]
        ) {
            Ok(_) => {return Ok(());}
            Err(err) => {
                error!("[{}]insert fleet carrier faction: {}",journal_id,err);
                return Err(err);
            }
        }
    }
    //Same for construction sites
    //{"Body": "Col 285 Sector HR-S b5-1 A 2", "Taxi": false, "event": "Location", "BodyID": 15, "Docked": true, "StarPos": [-31.4375, 64.40625, -241.25], "odyssey": true, "BodyType": "Planet", "Factions": [{"Name": "HIP 32135 Boys", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.06319, "Allegiance": "Independent", "Government": "Anarchy", "FactionState": "None"}, {"Name": "Nagii Union", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.581745, "Allegiance": "Independent", "Government": "Communism", "FactionState": "None", "PendingStates": [{"State": "Expansion", "Trend": 0}]}, {"Name": "HIP 32135 Conservatives", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.235707, "Allegiance": "Independent", "Government": "Dictatorship", "FactionState": "None"}, {"Name": "Weeb Alliance", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.119358, "Allegiance": "Independent", "Government": "Cooperative", "FactionState": "None"}], "MarketID": 4284827651, "horizons": true, "Multicrew": false, "timestamp": "2025-09-08T12:00:54Z", "Population": 2052372, "StarSystem": "Col 285 Sector HR-S b5-1", "StationName": "Planetary Construction Site: Jun Defence Installation", "StationType": "PlanetaryConstructionDepot", "SystemAddress": 2869441078601, "SystemEconomy": "$economy_Industrial;", "SystemFaction": {"Name": "Nagii Union"}, "DistFromStarLS": 14.962181, "StationEconomy": "$economy_Colony;", "StationFaction": {"Name": "Brewer Corporation"}, "SystemSecurity": "$SYSTEM_SECURITY_low;", "StationServices": ["dock", "autodock", "commodities", "contacts", "rearm", "refuel", "repair", "flightcontroller", "stationoperations", "stationMenu", "colonisationcontribution"], "StationEconomies": [{"Name": "$economy_Colony;", "Proportion": 1.0}], "SystemAllegiance": "Independent", "SystemGovernment": "$government_Communism;", "StationGovernment": "$government_Corporate;", "SystemSecondEconomy": "$economy_Extraction;"}
    if json["StationType"].to_string() == "PlanetaryConstructionDepot"
        || json["StationType"].to_string() == "SpaceConstructionDepot"
    {
        let allegiance = value_table(
            Tables::Allegiance,
            "PilotsFederation".to_string(),
            journal_id,
            client,
        )?;
        match client.execute(
            //language=postgresql
            "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                      government=$3,allegiance=$4,journal_id=$7",
            &[&faction_name,&system_address,&government,&allegiance,&happiness,&0.0f32,&journal_id]
        ) {
            Ok(_) => {return Ok(());}
            Err(err) => {
                error!("[{}]insert ConstructionDepot faction: {}",journal_id,err);
                return Err(err);
            }
        }
    }
    //And Megaships
    //{"Body": "The Iron Claw", "Taxi": false, "event": "Location", "BodyID": 13, "Docked": true, "StarPos": [-2.0, -1.46875, 99.6875], "odyssey": true, "BodyType": "Station", "MarketID": 128828823, "horizons": true, "Multicrew": false, "timestamp": "2025-09-08T12:25:49Z", "Population": 0, "StarSystem": "Scorpii Sector DB-X b1-8", "StationName": "The Iron Claw", "StationType": "MegaShip", "SystemAddress": 18263140541905, "SystemEconomy": "$economy_Extraction;", "SystemFaction": {"Name": "Independent Detention Foundation"}, "DistFromStarLS": 12928.471765, "StationEconomy": "$economy_Prison;", "StationFaction": {"Name": "Independent Detention Foundation"}, "SystemSecurity": "$SYSTEM_SECURITY_high;", "StationServices": ["dock", "autodock", "contacts", "outfitting", "rearm", "refuel", "repair", "shipyard", "engineer", "flightcontroller", "stationoperations", "stationMenu", "livery", "socialspace"], "StationEconomies": [{"Name": "$economy_Prison;", "Proportion": 1.0}], "SystemAllegiance": "Independent", "SystemGovernment": "$government_Prison;", "StationGovernment": "$government_Prison;", "SystemSecondEconomy": "$economy_None;"}
    if json["StationType"].to_string() == "MegaShip" {
        match client.execute(
            //language=postgresql
            "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                      government=$3,allegiance=$4,journal_id=$7",
            &[&faction_name,&system_address,&government,&system_allegiance,&happiness,&0.0f32,&journal_id]
        ) {
            Ok(_) => {return Ok(());}
            Err(err) => {
                error!("[{}]insert megaship faction: {}",journal_id,err);
                return Err(err);
            }
        }
    }
    //And Engineers and rest
    //{"Body": "Arque 4 e", "Taxi": false, "event": "Location", "BodyID": 28, "Docked": true, "Powers": ["Edmund Mahon", "Felicia Winters", "Nakato Kaine"], "StarPos": [66.5, 38.0625, 61.125], "odyssey": true, "BodyType": "Planet", "Factions": [{"Name": "Arque Commodities", "Happiness": "$Faction_HappinessBand3;", "Influence": 0.028028, "Allegiance": "Alliance", "Government": "Corporate", "ActiveStates": [{"State": "Lockdown"}, {"State": "Famine"}], "FactionState": "Lockdown", "RecoveringStates": [{"State": "InfrastructureFailure", "Trend": 0}]}, {"Name": "Uniting Arque", "Happiness": "$Faction_HappinessBand3;", "Influence": 0.036036, "Allegiance": "Alliance", "Government": "Cooperative", "ActiveStates": [{"State": "Lockdown"}, {"State": "Bust"}], "FactionState": "Lockdown", "RecoveringStates": [{"State": "InfrastructureFailure", "Trend": 0}]}, {"Name": "The Dark Wheel", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.181181, "Allegiance": "Independent", "Government": "Democracy", "FactionState": "None", "PendingStates": [{"State": "Expansion", "Trend": 0}], "RecoveringStates": [{"State": "InfrastructureFailure", "Trend": 0}]}, {"Name": "Arque Blue Posse", "Happiness": "$Faction_HappinessBand2;", "Influence": 0.01001, "Allegiance": "Independent", "Government": "Anarchy", "FactionState": "None"}, {"Name": "Federal Reclamation Co", "Happiness": "$Faction_HappinessBand3;", "Influence": 0.11011, "Allegiance": "Federation", "Government": "Corporate", "ActiveStates": [{"State": "InfrastructureFailure"}, {"State": "Retreat"}, {"State": "Expansion"}], "FactionState": "Retreat", "PendingStates": [{"State": "CivilUnrest", "Trend": 0}]}, {"Name": "Alliance Rapid-reaction Corps", "Happiness": "$Faction_HappinessBand1;", "Influence": 0.634635, "Allegiance": "Alliance", "Government": "Cooperative", "ActiveStates": [{"State": "Investment"}, {"State": "InfrastructureFailure"}, {"State": "Expansion"}], "FactionState": "Expansion"}], "MarketID": 128679559, "horizons": true, "Multicrew": false, "timestamp": "2025-07-17T15:21:08Z", "Population": 111849, "StarSystem": "Arque", "StationName": "Abel Laboratory", "StationType": "CraterPort", "SystemAddress": 113573366131, "SystemEconomy": "$economy_Extraction;", "SystemFaction": {"Name": "Alliance Rapid-reaction Corps", "FactionState": "Expansion"}, "DistFromStarLS": 2399.305388, "PowerplayState": "Fortified", "StationEconomy": "$economy_Colony;", "StationFaction": {"Name": "Professor Palin"}, "SystemSecurity": "$SYSTEM_SECURITY_low;", "StationServices": ["dock", "autodock", "commodities", "contacts", "exploration", "outfitting", "crewlounge", "rearm", "refuel", "repair", "tuning", "engineer", "facilitator", "flightcontroller", "stationoperations", "searchrescue", "stationMenu", "shop", "livery"], "ControllingPower": "Edmund Mahon", "StationEconomies": [{"Name": "$economy_Colony;", "Proportion": 1.0}], "SystemAllegiance": "Alliance", "SystemGovernment": "$government_Cooperative;", "StationGovernment": "$government_Engineer;", "SystemSecondEconomy": "$economy_Agri;", "PowerplayStateUndermining": 0, "PowerplayStateReinforcement": 0, "PowerplayStateControlProgress": 0.165846}
    match client.execute(
        //language=postgresql
        "INSERT INTO factions (name, system_address, government, allegiance, happiness, influence, journal_id) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
                                                                                                      government=$3,allegiance=$4,journal_id=$7",
        &[&faction_name,&system_address,&government,&system_allegiance,&happiness,&0.0f32,&journal_id]
    ) {
        Ok(_) => {Ok(())}
        Err(err) => {
            error!("[{}]insert station faction: {}",journal_id,err);
            Err(err)
        }
    }
}

pub fn insert_factions(
    json: &JsonValue,
    client: &mut Client,
    system_address: &i64,
    journal_id: i64,
) -> Result<(), Error> {
    match client.execute(
        // language=postgresql
        "DELETE FROM faction_states WHERE system_address = $1",
        &[&system_address],
    ) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "[{}]insert_faction_state: Couldn't delete old faction state from system {}: {}",
                journal_id, system_address, err
            );
        }
    }
    let faction_array_size = json["Factions"].len();
    for i in 0..faction_array_size {
        let json = &json["Factions"][i];
        let faction_name_id = value_table(
            Tables::FactionName,
            json["Name"].to_string(),
            journal_id,
            client,
        )?;
        let government = value_table(
            Tables::Government,
            json["Government"].to_string(),
            journal_id,
            client,
        )?;
        let allegiance = value_table(
            Tables::Allegiance,
            json["Allegiance"].to_string(),
            journal_id,
            client,
        )?;
        let happiness = value_table(
            Tables::Happiness,
            json["Happiness"].to_string(),
            journal_id,
            client,
        )?;
        let influence = json["Influence"]
            .as_f32()
            .expect("Couldn't parse influence in insert faction");

        let faction_key = insert_faction(
            system_address.clone(),
            faction_name_id,
            government,
            allegiance,
            happiness,
            influence,
            journal_id,
            client,
        )?;
        let state_size = json["PendingStates"].len();
        for j in 0..state_size {
            let json = &json["PendingStates"][j];
            let mut trend = None;
            if json.has_key("Trend") {
                trend = json["Trend"].as_f32();
            }
            let state_name = value_table(
                Tables::FactionStateName,
                json["State"].to_string(),
                journal_id,
                client,
            )?;
            insert_faction_state(
                faction_key.0,
                faction_key.1,
                state_name,
                FactionStateState::Pending,
                trend,
                journal_id,
                client,
            )?;
        }
        let state_size = json["ActiveStates"].len();
        for j in 0..state_size {
            let json = &json["ActiveStates"][j];
            let mut trend = None;
            if json.has_key("Trend") {
                trend = json["Trend"].as_f32();
            }
            let state_name = value_table(
                Tables::FactionStateName,
                json["State"].to_string(),
                journal_id,
                client,
            )?;
            insert_faction_state(
                faction_key.0,
                faction_key.1,
                state_name,
                FactionStateState::Active,
                trend,
                journal_id,
                client,
            )?;
        }
        let state_size = json["RecoveringStates"].len();
        for j in 0..state_size {
            let json = &json["RecoveringStates"][j];
            let mut trend = None;
            if json.has_key("Trend") {
                trend = json["Trend"].as_f32();
            }
            let state_name = value_table(
                Tables::FactionStateName,
                json["State"].to_string(),
                journal_id,
                client,
            )?;
            insert_faction_state(
                faction_key.0,
                faction_key.1,
                state_name,
                FactionStateState::Recovering,
                trend,
                journal_id,
                client,
            )?;
        }
    }
    Ok(())
}

fn insert_faction(
    system_address: i64,
    faction_name: i32,
    government: i32,
    allegiance: i32,
    happiness: i32,
    influence: f32,
    journal_id: i64,
    client: &mut Client,
) -> Result<(i32, i64), Error> {
    let faction_keys: Option<(i32, i64)> = match client.query_one(
        // language=postgresql
        "SELECT name,system_address FROM factions WHERE system_address=$1 AND name=$2",
        &[&system_address, &faction_name],
    ) {
        Ok(row) => {
            if row.is_empty() {
                None
            } else {
                Some((row.get(0), row.get(1)))
            }
        }
        Err(err) => {
            if err.to_string() != "query returned an unexpected number of rows" {
                error!(
                    "[{}]insert_faction: Unable to get faction: {}",
                    journal_id, err
                );
                return Err(err);
            }
            None
        }
    };
    match faction_keys {
        None => {
            //Insert new faction
            match client.query_one(
                // language=postgresql
                "INSERT INTO factions
                    (name, system_address, government, influence, allegiance, happiness,journal_id)
                    VALUES
                    ($1,$2,$3,$4,$5,$6,$7)
                    RETURNING name,system_address",
                &[
                    &faction_name,
                    &system_address,
                    &government,
                    &influence,
                    &allegiance,
                    &happiness,
                    &journal_id,
                ],
            ) {
                Ok(row) => Ok((row.get(0), row.get(1))),
                Err(err) => {
                    error!(
                        "[{}]insert_factions: Unable to insert faction: {}",
                        journal_id, err
                    );
                    Err(err)
                }
            }
        }
        Some(faction_keys) => {
            //Update faction state
            match client.execute(
                // language=postgresql
                "
                    UPDATE factions
                    SET
                        government=$3,
                        influence=$4,
                        allegiance=$5,
                        happiness=$6,
                        journal_id=$7

                    WHERE system_address=$2 AND name=$1",
                &[
                    &faction_name,
                    &system_address,
                    &government,
                    &influence,
                    &allegiance,
                    &happiness,
                    &journal_id,
                ],
            ) {
                Ok(_) => Ok(faction_keys),
                Err(err) => {
                    error!(
                        "[{}]insert_factions: Unable to update factions: {}",
                        journal_id, err
                    );
                    Err(err)
                }
            }
        }
    }
}

#[derive(Debug, postgres_types::ToSql, postgres_types::FromSql)]
#[postgres(name = "faction_state_state", rename_all = "snake_case")]
enum FactionStateState {
    #[postgres(name = "pending")]
    Pending,
    #[postgres(name = "active")]
    Active,
    #[postgres(name = "recovering")]
    Recovering,
}

fn insert_faction_state(
    faction_name: i32,
    system_address: i64,
    state_name: i32,
    state_state: FactionStateState,
    trend: Option<f32>,
    journal_id: i64,
    client: &mut Client,
) -> Result<(), Error> {
    match client.execute(
        //language=postgresql
        "INSERT INTO faction_states (faction, system_address, state_name, state_state, trend,journal_id) VALUES ($1, $2, $3, $4, $5, $6)",
        &[&faction_name, &system_address, &state_name, &state_state, &trend, &journal_id])
    {
        Ok(_) => {Ok(())}
        Err(err) => {
            error!("[{}]insert_faction_state: Couldn't insert new faction state: {}",journal_id,err);
            Err(err)
        }
    }
}

pub fn insert_conflict(
    json: &JsonValue,
    client: &mut Client,
    system_address: &i64,
    journal_id: i64,
) -> Result<(), Error> {
    match client.execute(
        //language=postgresql
        "DELETE FROM conflicts WHERE system_address = $1",
        &[&system_address],
    ) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "[{}]insert_conflict: Couldn't delete conflict table: {}",
                journal_id, err
            );
            return Err(err);
        }
    }
    match client.execute(
        //language=postgresql
        "DELETE FROM conflict_faction_status WHERE system_address = $1",
        &[&system_address],
    ) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "[{}]insert_conflict: Couldn't delete conflict_faction_Status table: {}",
                journal_id, err
            );
            return Err(err);
        }
    }

    let conflict_size = json["Conflicts"].len();
    for i in 0..conflict_size {
        let json = &json["Conflicts"][i];
        let war_type = value_table(
            Tables::WarType,
            json["WarType"].to_string(),
            journal_id,
            client,
        )?;
        let status = value_table(
            Tables::ConflictStatus,
            json["Status"].to_string(),
            journal_id,
            client,
        )?;
        let faction_one_name = value_table(
            Tables::FactionName,
            json["Faction1"]["Name"].to_string(),
            journal_id,
            client,
        )?;
        let faction_one: i32 = match client.query_one(
            //language=postgresql
            "INSERT INTO conflict_faction_status(stake, won_days, name, system_address,journal_id) VALUES ($1, $2, $3, $4,$5) RETURNING id",
            &[&json["Faction1"]["Stake"].to_string(), &json["Faction1"]["WonDays"].as_i32().unwrap(), &faction_one_name, &system_address,&journal_id]
        ) {
            Ok(row) => { row.get(0) }
            Err(err) => {
                error!("[{}]insert_conflict: Failed to insert faction one: {}",journal_id,err);
                return Err(err);
            }
        };
        let faction_two_name = value_table(
            Tables::FactionName,
            json["Faction2"]["Name"].to_string(),
            journal_id,
            client,
        )?;
        let faction_two: i32 = match client.query_one(
            //language=postgresql
            "INSERT INTO conflict_faction_status(stake, won_days, name, system_address,journal_id) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&json["Faction2"]["Stake"].to_string(), &json["Faction2"]["WonDays"].as_i32().unwrap(), &faction_two_name, &system_address, &journal_id]
        ) {
            Ok(row) => { row.get(0) }
            Err(err) => {
                error!("[{}]Location: Failed to insert faction two: {}",journal_id,err);
                return Err(err);
            }
        };
        match client.execute(
            //language=postgresql
            "INSERT INTO conflicts (system_address, faction1, faction2, war_type, status, journal_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&system_address, &faction_one, &faction_two, &war_type, &status, &journal_id]
        ) {
            Ok(_) => {}
            Err(err) => {
                error!("[{}]Location: Failed to insert conflict: {}",journal_id,err);
            }
        }
    }
    Ok(())
}
