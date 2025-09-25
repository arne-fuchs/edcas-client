use serde::{Deserialize, Serialize};

use crate::edcas::assets::faction::{conflict::Conflict, Faction, SystemFaction};

#[derive(Serialize, Deserialize)]
pub struct Fsdjump {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "SystemSecondEconomy")]
    system_second_economy: String,

    #[serde(rename = "SystemGovernment")]
    system_government: String,

    #[serde(rename = "BodyType")]
    body_type: String,

    #[serde(rename = "SystemAllegiance")]
    system_allegiance: String,

    #[serde(rename = "SystemEconomy")]
    system_economy: String,

    #[serde(rename = "Population")]
    population: i64,

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

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<Conflict>>,

    #[serde(rename = "Multicrew")]
    #[serde(default)]
    multicrew: bool,

    #[serde(rename = "SystemFaction")]
    system_faction: Option<SystemFaction>,

    #[serde(rename = "BodyID")]
    body_id: i32,

    #[serde(rename = "Powers")]
    powers: Option<Vec<String>>,

    #[serde(rename = "ControllingPower")]
    controlling_power: Option<String>,

    #[serde(rename = "PowerplayStateUndermining")]
    powerplay_state_undermining: Option<i32>,

    #[serde(rename = "PowerplayStateReinforcement")]
    powerplay_state_reinforcement: Option<i32>,

    #[serde(rename = "PowerplayStateControlProgress")]
    powerplay_state_control_progress: Option<f32>,
}

impl Fsdjump {
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
            system_government,
            body_type,
            system_allegiance,
            system_economy,
            population,
            event: _,
            body,
            timestamp,
            star_pos,
            system_security,
            factions,
            star_system,
            conflicts,
            multicrew: _,
            system_faction,
            body_id,
            powers,
            controlling_power,
            powerplay_state_undermining,
            powerplay_state_reinforcement,
            powerplay_state_control_progress,
        } = self;
        //TODO Missing: Thargoid war status. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        //TODO Power play status is mssing. See https://elite-journal.readthedocs.io/en/latest/Travel.html#fsdjump
        // "PowerplayStateUndermining": 4110,
        // "PowerplayStateReinforcement": 12725,
        // "PowerplayStateControlProgress": 0.056415
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
        if let Some(system_faction) = system_faction {
            system_faction.insert_into_db(journal_id, client)?;
        }
        let system_allegiance =
            value_table(Tables::Allegiance, system_allegiance, journal_id, client)?;
        let economy = value_table(Tables::EconomyType, system_economy, journal_id, client)?;
        let second_economy = value_table(
            Tables::EconomyType,
            system_second_economy,
            journal_id,
            client,
        )?;
        let government = value_table(Tables::Government, system_government, journal_id, client)?;
        let security = value_table(Tables::Security, system_security, journal_id, client)?;

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

        if let Some(factions) = factions {
            for faction in factions {
                faction.insert_into_db(journal_id, system_address, client)?;
            }
        }
        if let Some(conflicts) = conflicts {
            for conflict in conflicts {
                conflict.insert_into_db(journal_id, system_address, client)?;
            }
        }

        Ok(())
    }
}
