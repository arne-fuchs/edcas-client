use serde::{Deserialize, Serialize};

#[cfg(feature = "eddn")]
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

#[derive(Serialize, Deserialize)]
pub struct ActiveState {
    #[serde(rename = "State")]
    state: String,
}
impl ActiveState {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        faction_name: i32,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self { state } = self;

        let state = value_table(Tables::FactionStateName, state, journal_id, client)?;

        match client.execute(
            //language=postgresql
            "INSERT INTO faction_states (faction, system_address, state_name, state_state,journal_id) VALUES ($1, $2, $3, $4, $5)",
            &[&faction_name, &system_address,  &state, &FactionStateState::Active, &journal_id])
        {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("[{}]insert_faction_state: Couldn't insert new faction state: {}",journal_id,err);
                Err(err)
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct PendingState {
    #[serde(rename = "State")]
    state: String,

    #[serde(rename = "Trend")]
    trend: f32,
}
impl PendingState {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        faction_name: i32,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self { state, trend } = self;

        let state = value_table(Tables::FactionStateName, state, journal_id, client)?;
        match client.execute(
            //language=postgresql
            "INSERT INTO faction_states (faction, system_address, state_name, state_state, trend,journal_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&faction_name, &system_address, &state, &FactionStateState::Pending, &trend, &journal_id])
        {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("[{}]insert_faction_state: Couldn't insert new faction state: {}",journal_id,err);
                Err(err)
            }
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct RecoveringState {
    #[serde(rename = "State")]
    state: String,

    #[serde(rename = "Trend")]
    trend: f32,
}
impl RecoveringState {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        faction_name: i32,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self { state, trend } = self;

        let state = value_table(Tables::FactionStateName, state, journal_id, client)?;
        match client.execute(
            //language=postgresql
            "INSERT INTO faction_states (faction, system_address, state_name, state_state, trend,journal_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&faction_name, &system_address, &state, &FactionStateState::Recovering, &trend, &journal_id])
        {
            Ok(_) => Ok(()),
            Err(err) => {
                log::error!("[{}]insert_faction_state: Couldn't insert new faction state: {}",journal_id,err);
                Err(err)
            }
        }
    }
}
