use log::error;
use postgres::{Client, Error};

pub fn insert_star_system(
    system_address: i64,
    name: String,
    position: (f32, f32, f32),
    allegiance: i32,
    economy: i32,
    second_economy: i32,
    government: i32,
    security: i32,
    population: i64,
    controlling_power: Option<i32>,
    journal_id: i64,
    client: &mut Client,
) -> Result<i64, Error> {
    let system_address_db: Option<i64> = match client.query_one(
        // language=postgresql
        "SELECT system_address FROM star_systems WHERE system_address=$1",
        &[&system_address],
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
                error!(
                    "[{}]insert_star_system: Unable to get star system: {}",
                    journal_id, err
                );
                return Err(err);
            }
            None
        }
    };

    match system_address_db {
        None => {
            //Doesn't exist -> Insert
            match client.query_one(
                // language=postgresql
                "INSERT INTO star_systems
                    (system_address,name,x,y,z,allegiance,economy,second_economy,government,security,population,controlling_power,journal_id)
                    VALUES
                    ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
                    RETURNING system_address",
                &[&system_address,&name,&position.0,&position.1,&position.2,&allegiance,&economy,&second_economy,&government,&security,&population,&controlling_power,&journal_id])
            {
                Ok(row) => Ok(row.get(0)),
                Err(err) => {
                    error!("[{}]insert_star_system: Unable to insert star system: {}",journal_id,err);
                    Err(err)
                }
            }
        }
        Some(system_address) => {
            //Exists -> Update
            match client.execute(
                // language=postgresql
                "
                    UPDATE star_systems
                    SET
                        name=$1,
                        x=$2,
                        y=$3,
                        z=$4,
                        allegiance=$5,
                        economy=$6,
                        second_economy=$7,
                        government=$8,
                        security=$9,
                        population=$10,
                        controlling_power=$11,
                        journal_id=$12
                    WHERE system_address=$13",
                &[
                    &name,
                    &position.0,
                    &position.1,
                    &position.2,
                    &allegiance,
                    &economy,
                    &second_economy,
                    &government,
                    &security,
                    &population,
                    &controlling_power,
                    &journal_id,
                    &system_address,
                ],
            ) {
                Ok(_) => Ok(system_address),
                Err(err) => {
                    error!(
                        "[{}]insert_star_system: Unable to update star systems: {}",
                        journal_id, err
                    );
                    Err(err)
                }
            }
        }
    }
}
