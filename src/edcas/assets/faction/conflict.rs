use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Conflict {
    #[serde(rename = "WarType")]
    war_type: String,

    #[serde(rename = "Status")]
    status: String,

    #[serde(rename = "Faction2")]
    faction2: ConflictStatus,

    #[serde(rename = "Faction1")]
    faction1: ConflictStatus,
}

impl Conflict {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        client: &mut postgres::Client,
    ) -> Result<(), postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};

        let Self {
            war_type,
            status,
            faction2,
            faction1,
        } = self;
        let war_type = value_table(Tables::WarType, war_type, journal_id, client)?;
        let status = value_table(Tables::ConflictStatus, status, journal_id, client)?;

        let faction1 = faction1.insert_into_db(journal_id, system_address, client)?;

        let faction2 = faction2.insert_into_db(journal_id, system_address, client)?;

        match client.execute(
            //language=postgresql
            "INSERT INTO conflicts (system_address, faction1, faction2, war_type, status, journal_id) VALUES ($1, $2, $3, $4, $5, $6)",
            &[&system_address, &faction1, &faction2, &war_type, &status, &journal_id]
        ) {
            Ok(_) =>  Ok(()),
            Err(err) => {
                log::error!("[{}]Location: Failed to insert conflict: {}",journal_id,err);
                Err(err)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ConflictStatus {
    #[serde(rename = "WonDays")]
    won_days: i32,

    #[serde(rename = "Stake")]
    stake: String,

    #[serde(rename = "Name")]
    name: String,
}

impl ConflictStatus {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        client: &mut postgres::Client,
    ) -> Result<i32, postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};
        let Self {
            won_days,
            stake,
            name,
        } = self;
        let name = value_table(Tables::FactionName, name, journal_id, client)?;

        match client.query_one(
            //language=postgresql
            "INSERT INTO conflict_faction_status(stake, won_days, name, system_address,journal_id) VALUES ($1, $2, $3, $4,$5) RETURNING id",
            &[&stake, &won_days, &name, &system_address,&journal_id]
        ) {
            Ok(row) => Ok(row.get(0)),
            Err(err) => {
                log::error!("[{}]insert_conflict: Failed to insert faction one: {}",journal_id,err);
                return Err(err);
            }
        }
    }
}
