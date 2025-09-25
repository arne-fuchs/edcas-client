use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StationEconomy {
    #[serde(alias = "Proportion")]
    #[serde(alias = "proporation")]
    proportion: f32,

    #[serde(alias = "Name")]
    #[serde(alias = "name")]
    name: String,
}
//TODO Own table implementation with reference to market id. Stations are able to have several economies
impl StationEconomy {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        //TODO: Implement
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct StationFaction {
    #[serde(rename = "Name")]
    name: String,
}
impl StationFaction {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Transaction,
    ) -> Result<i32, postgres::Error> {
        use crate::edcas::tables::{value_table, Tables};

        let faction_name = value_table(Tables::FactionName, self.name, journal_id, client)?;

        Ok(faction_name)
    }
}

#[derive(Serialize, Deserialize)]
pub struct LandingPads {
    #[serde(rename = "Small")]
    small: i64,

    #[serde(rename = "Medium")]
    medium: i64,

    #[serde(rename = "Large")]
    large: i64,
}
impl LandingPads {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        market_id: i64,
        client: &mut postgres::Transaction,
    ) -> Result<(), postgres::Error> {
        //TODO: Implement
        Ok(())
    }
}
