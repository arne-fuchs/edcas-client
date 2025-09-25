use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Star {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "Radius")]
    radius: f32,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "AxialTilt")]
    axial_tilt: f32,

    #[serde(rename = "Subclass")]
    subclass: i32,

    #[serde(rename = "StarType")]
    star_type: String,

    #[serde(rename = "Luminosity")]
    luminosity: String,

    #[serde(rename = "Age_MY")]
    age_my: i32,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "WasMapped")]
    was_mapped: bool,

    #[serde(rename = "RotationPeriod")]
    rotation_period: f32,

    #[serde(rename = "DistanceFromArrivalLS")]
    distance_from_arrival_ls: f32,

    #[serde(rename = "odyssey")]
    #[serde(default)]
    odyssey: bool,

    #[serde(rename = "horizons")]
    #[serde(default)]
    horizons: bool,

    #[serde(rename = "ScanType")]
    scan_type: String,

    #[serde(rename = "StellarMass")]
    stellar_mass: f32,

    #[serde(rename = "BodyID")]
    body_id: i32,

    #[serde(rename = "SurfaceTemperature")]
    surface_temperature: f32,

    #[serde(rename = "Rings")]
    rings: Option<Vec<super::body::Ring>>,

    #[serde(rename = "AbsoluteMagnitude")]
    absolute_magnitude: f32,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "BodyName")]
    body_name: String,

    #[serde(rename = "WasDiscovered")]
    was_discovered: bool,

    #[serde(rename = "timestamp")]
    timestamp: String,
}
impl Star {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::{edcas::tables::value_table, eddn::edcas_error::EdcasError};

        let Self {
            system_address,
            radius,
            star_pos,
            axial_tilt,
            subclass,
            star_type,
            luminosity,
            age_my,
            star_system,
            was_mapped: _,
            rotation_period,
            distance_from_arrival_ls,
            odyssey: _,
            horizons: _,
            scan_type: _,
            stellar_mass,
            body_id,
            surface_temperature,
            rings,
            absolute_magnitude,
            event: _,
            body_name,
            was_discovered: _,
            timestamp,
        } = self;
        //TODO: Implement
        let _ = timestamp;
        //TODO Does this even work?
        if let Err(err) = client.execute(
            //language_postgres
            "INSERT INTO star_systems (system_address,name,x,y,z,journal_id) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
            &[&system_address,&star_system,&star_pos[0],&star_pos[1],&star_pos[2],&journal_id]
        ){
            return Err(EdcasError::new(format!("[Star]: insert system: {}", err)));
        }

        let star_type = value_table(
            crate::edcas::tables::Tables::StarType,
            star_type,
            journal_id,
            client,
        )?;

        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO star (id, system_address, name, age_my, radius, star_type, subclass, axial_tilt, luminosity, stellar_mass,
            rotation_period, absolute_magnitude, surface_temperature, distance, journal_id) VALUES
            ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15) ON CONFLICT ON CONSTRAINT star_pkey DO UPDATE SET
            name=$3,age_my=$4,radius=$5,star_type=$6,subclass=$7,axial_tilt=$8,luminosity=$9,stellar_mass=$10,rotation_period=$11,absolute_magnitude=$12,surface_temperature=$13,distance=$14,journal_id=$15",
            &[&body_id,&system_address,&body_name,&age_my,&radius,&star_type,&subclass,&axial_tilt,&luminosity,&stellar_mass,&rotation_period,&absolute_magnitude,&surface_temperature,&distance_from_arrival_ls,&journal_id]
        ) {
            log::error!("[Star] inserting star: {}",err);
            return Err(EdcasError::from(err));
        }
        if let Some(rings) = rings {
            for ring in rings {
                ring.insert_into_db(journal_id, system_address, body_id, client)?;
            }
        }
        Ok(())
    }
}
