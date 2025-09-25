use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Ships {
    #[serde(rename = "ships")]
    ships: Vec<String>,

    #[serde(rename = "systemName")]
    system_name: String,

    #[serde(rename = "stationName")]
    station_name: String,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "marketId")]
    market_id: i64,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "timestamp")]
    timestamp: String,
}
impl Ships {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            ships,
            system_name,
            station_name,
            horizons,
            market_id,
            odyssey,
            timestamp,
        } = self;
        //TODO: Implement
        let mut transaction = client.transaction()?;
        if let Err(err) = transaction.execute(
            //language=postgresql
            "DELETE FROM ship_listening WHERE market_id=$1",
            &[&market_id],
        ) {
            return Err(EdcasError::new(format!(
                "Couldn't delete old ship listenings: {}",
                err
            )));
        }
        for ship in ships {
            let ship_name = value_table(Tables::ShipName, ship, journal_id, &mut transaction)?;
            if let Err(err) = transaction.execute(
                //language=postgresql
                "INSERT INTO ship_listening (ship_name, market_id, journal_id) VALUES ($1,$2,$3)",
                &[&ship_name, &market_id, &journal_id],
            ) {
                return Err(EdcasError::new(format!(
                    "Couldn't insert ship listening: {}",
                    err
                )));
            }
        }
        transaction.commit()?;
        Ok(())
    }
}
