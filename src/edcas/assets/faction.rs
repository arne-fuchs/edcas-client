use serde::{Deserialize, Serialize};

pub mod conflict;
pub mod state;

#[derive(Serialize, Deserialize)]
pub struct SystemFaction {
    #[serde(rename = "FactionState")]
    faction_state: Option<String>,

    #[serde(rename = "Name")]
    name: String,
}

impl SystemFaction {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self {
            faction_state,
            name,
        } = self;
        value_table(Tables::FactionName, name, journal_id, client)?;
        if let Some(faction_state) = faction_state {
            value_table(Tables::FactionStateName, faction_state, journal_id, client)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Faction {
    #[serde(rename = "Happiness")]
    happiness: String,

    #[serde(rename = "Allegiance")]
    allegiance: String,

    #[serde(rename = "Government")]
    government: String,

    #[serde(rename = "Influence")]
    influence: f32,

    #[serde(rename = "FactionState")]
    faction_state: String,

    #[serde(rename = "Name")]
    name: String,

    #[serde(rename = "MyReputation")]
    my_reputation: Option<f32>,

    #[serde(rename = "PendingStates")]
    pending_states: Option<Vec<state::PendingState>>,

    #[serde(rename = "ActiveStates")]
    active_states: Option<Vec<state::ActiveState>>,

    #[serde(rename = "RecoveringStates")]
    recovering_states: Option<Vec<state::RecoveringState>>,
}

impl Faction {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self {
            happiness,
            allegiance,
            government,
            influence,
            faction_state,
            name,
            my_reputation: _,
            pending_states,
            active_states,
            recovering_states,
        } = self;
        let mut transaction = client.transaction()?;
        let faction_name_id = value_table(Tables::FactionName, name, journal_id, &mut transaction)?;
        let government = value_table(Tables::Government, government, journal_id, &mut transaction)?;
        let allegiance = value_table(Tables::Allegiance, allegiance, journal_id, &mut transaction)?;
        let happiness = value_table(Tables::Happiness, happiness, journal_id, &mut transaction)?;
        let _ = value_table(Tables::FactionStateName, faction_state, journal_id, &mut transaction)?;
        transaction.commit()?;

        let name = insert_faction(
            system_address,
            faction_name_id,
            government,
            allegiance,
            happiness,
            influence,
            journal_id,
            client,
        )?;
        let mut transaction = client.transaction()?;
        if let Err(err) = transaction.execute(
            "DELETE FROM faction_states WHERE system_address=$1 AND faction=$2",
            &[&system_address, &faction_name_id],
        ) {
            log::error!(
                "remove old faction state: Unable to delete old states: {}",
                err
            );
            return Err(err);
        }
        if let Some(active_states) = active_states {
            for active_state in active_states {
                active_state.insert_into_db(journal_id, system_address, name, &mut transaction)?;
            }
        }
        if let Some(pending_states) = pending_states {
            for pending_state in pending_states {
                pending_state.insert_into_db(journal_id, system_address, name, &mut transaction)?;
            }
        }
        if let Some(recovering_states) = recovering_states {
            for recovering_states in recovering_states {
                recovering_states.insert_into_db(journal_id, system_address, name, &mut transaction)?;
            }
        }
        transaction.commit()?;
        Ok(())
    }
}

#[cfg(feature = "eddn")]
fn insert_faction(
    system_address: i64,
    faction_name: i32,
    government: i32,
    allegiance: i32,
    happiness: i32,
    influence: f32,
    journal_id: i64,
    client: &mut postgres::Transaction,
) -> Result<i32, postgres::Error> {
    let faction_key: Option<i32> = match client.query_one(
        // language=postgresql
        "SELECT name FROM factions WHERE system_address=$1 AND name=$2",
        &[&system_address, &faction_name],
    ) {
        Ok(row) => {
            if row.is_empty() {
                None
            } else {
                Some(row.get(0))
            }
        }
        Err(err) => {
            if err.to_string() != "query returned an unexpected number of rows" {
                log::error!("insert_faction: Unable to get faction: {}", err);
                return Err(err);
            }
            None
        }
    };
    match faction_key {
        None => {
            //Insert new faction
            match client.query_one(
                // language=postgresql
                "INSERT INTO factions
                    (name, system_address, government, influence, allegiance, happiness,journal_id)
                    VALUES
                    ($1,$2,$3,$4,$5,$6,$7)
                    RETURNING name",
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
                Ok(row) => Ok(row.get(0)),
                Err(err) => {
                    log::error!(
                        "[{}]insert_factions: Unable to insert faction: {}",
                        journal_id,
                        err
                    );
                    Err(err)
                }
            }
        }
        Some(faction_key) => {
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
                Ok(_) => Ok(faction_key),
                Err(err) => {
                    log::error!(
                        "[{}]insert_factions: Unable to update factions: {}",
                        journal_id,
                        err
                    );
                    Err(err)
                }
            }
        }
    }
}
