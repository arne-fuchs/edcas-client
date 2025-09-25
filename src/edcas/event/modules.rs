use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Modules {
    #[serde(rename = "systemName")]
    system_name: String,

    #[serde(rename = "stationName")]
    station_name: String,

    #[serde(rename = "modules")]
    modules: Vec<String>,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "marketId")]
    market_id: i64,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "timestamp")]
    timestamp: String,
}
impl Modules {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            system_name,
            station_name,
            modules,
            horizons,
            market_id,
            odyssey,
            timestamp,
        } = self;

        if let Err(err) = client.execute(
            //language=postgresql
            "DELETE FROM modul_listening WHERE market_id=$1",
            &[&market_id],
        ) {
            return Err(EdcasError::new(format!(
                "Couldn't delete old modul listenings: {}",
                err
            )));
        }

        for module in modules {
            let name = value_table(Tables::ModulName, module, journal_id, client)?;
            if let Err(err) = client.execute(
                //language=postgresql
                "INSERT INTO modul_listening (modul_name, market_id, journal_id) VALUES ($1,$2,$3)",
                &[&name, &market_id, &journal_id],
            ) {
                return Err(EdcasError::new(format!(
                    "Couldn't insert modul listenings: {}",
                    err
                )));
            }
        }
        Ok(())
    }
}
