use serde::{Deserialize, Serialize};

//TODO seperate body from rings, beltclusters etc.

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Body {
    system_address: Option<i64>,
    radius: Option<f32>,
    axial_tilt: Option<f32>,
    orbital_period: Option<f32>,
    eccentricity: Option<f32>,
    parents: Vec<Parent>,
    materials: Option<Vec<Materials>>,
    surface_pressure: Option<f32>,
    orbital_inclination: Option<f32>,//Belt cluster do not have this
    rotation_period: Option<f32>,
    #[serde(rename = "MassEM")]
    #[serde(default)]
    mass_em: f32,//Rings do not have massem so default
    #[serde(default)]
    terraform_state: String,
    #[serde(rename = "DistanceFromArrivalLS")]
    distance_from_arrival_ls: f32,
    #[serde(rename = "odyssey")]
    #[serde(default)]
    odyssey: bool,
    semi_major_axis: Option<f32>,//Belt cluster do not have this
    surface_gravity: Option<f32>,//Belt cluster do not have this
    surface_temperature: Option<f32>,//Belt cluster do not have this
    ascending_node: Option<f32>,//Belt cluster do not have this
    atmosphere_composition: Option<Vec<AtmosphereComposition>>,
    #[serde(rename = "event")]
    event: String,
    #[serde(default)]
    landable: bool,
    volcanism: String,
    was_discovered: bool,
    #[serde(rename = "timestamp")]
    timestamp: String,
    star_pos: Vec<f32>,
    composition: Option<Composition>,
    atmosphere_type: Option<String>,
    star_system: String,
    was_mapped: bool,
    planet_class: String,
    #[serde(rename = "horizons")]
    #[serde(default)]
    horizons: bool,
    scan_type: String,
    mean_anomaly: f32,
    #[serde(rename = "BodyID")]
    body_id: i32,
    atmosphere: String,//Belt cluster do not have this
    periapsis: Option<f32>,//Belt cluster do not have this
    body_name: String,
    tidal_lock: bool,
}
impl Body {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::{
            edcas::tables::{value_table, Tables},
            eddn::edcas_error::EdcasError,
        };
        let Self {
            system_address,
            radius,
            axial_tilt,
            orbital_period,
            eccentricity,
            parents,
            materials,
            surface_pressure,
            orbital_inclination,
            rotation_period,
            mass_em,
            terraform_state,
            distance_from_arrival_ls,
            odyssey: _,
            semi_major_axis,
            surface_gravity,
            surface_temperature,
            ascending_node,
            atmosphere_composition,
            event: _,
            landable,
            volcanism,
            was_discovered: _,
            timestamp,
            star_pos,
            composition,
            atmosphere_type,
            star_system,
            was_mapped,
            planet_class,
            horizons: _,
            scan_type,
            mean_anomaly,
            body_id,
            atmosphere,
            periapsis,
            body_name,
            tidal_lock,
        } = self;
        //TODO Does this even work?
        let system_address = match system_address {
            Some(system_address) => {
                if let Err(err) = client.execute(
                    //language_postgres
                    "INSERT INTO star_systems (system_address,name,x,y,z,journal_id) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT DO NOTHING",
                    &[&system_address,&star_system,&star_pos[0],&star_pos[1],&star_pos[2],&journal_id]
                ){
                    return Err(EdcasError::new(format!("[Body] inserting star system: {}", err)));
                }
                system_address
            }
            None => {
                match client.query_one(
                    "SELECT system_address from star_systems where name=$1",
                    &[&star_system],
                ) {
                    Ok(row) => row.get(0),
                    Err(err) => return Err(EdcasError::from(err)),
                }
            }
        };

        let volcanism = value_table(Tables::Volcanism, volcanism, journal_id, client)?;
        let atmosphere = value_table(Tables::Atmosphere, atmosphere, journal_id, client)?;
        let planet_class = value_table(Tables::PlanetClass, planet_class, journal_id, client)?;
        let atmosphere_type = value_table(
            Tables::AtmosphereType,
            match atmosphere_type {
                Some(f) => f,
                None => "".to_string(),
            },
            journal_id,
            client,
        )?;
        let terraform_state =
            value_table(Tables::TerraformState, terraform_state, journal_id, client)?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO body
                (id, system_address, name, mass_em, radius, landable, axial_tilt, periapsis, tidal_lock, volcanism, mapped, atmosphere,
                mean_anomaly, planet_class, eccentricity, ascending_node, orbital_period, semi_major_axis, atmosphere_type, rotation_period,
                surface_gravity, terraform_state, surface_pressure, orbital_inclination, surface_temperature, distance,journal_id)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12, $13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27) ON CONFLICT ON CONSTRAINT body_pkey DO UPDATE SET
                mass_em=$4,radius=$5,landable=$6,axial_tilt=$7,periapsis=$8,tidal_lock=$9,volcanism=$10,mapped=$11,atmosphere=$12,mean_anomaly=$13,
                planet_class=$14,eccentricity=$15,ascending_node=$16,orbital_period=$17,semi_major_axis=$18,atmosphere_type=$19,rotation_period=$20,surface_gravity=$21,
                terraform_state=$22,surface_pressure=$23,orbital_inclination=$24,surface_temperature=$25,distance=$26,journal_id=$27",
            &[&body_id,&system_address,&body_name,&mass_em,&radius,&landable,&axial_tilt,&periapsis,&tidal_lock,&volcanism,&was_mapped,
                &atmosphere,&mean_anomaly,&planet_class,&eccentricity,&ascending_node,&orbital_period,&semi_major_axis,&atmosphere_type,&rotation_period,
                &surface_gravity,&terraform_state,&surface_pressure,&orbital_inclination,&surface_temperature,&distance_from_arrival_ls,&journal_id]
        ){
            log::error!("[Body][{}] inserting body: {}",scan_type, err);
            return Err(EdcasError::from(err));
        }
        if let Err(err) = client.execute(
            //language=postgresql
            "DELETE FROM atmosphere_composition WHERE body_id=$1 AND system_address=$2",
            &[&body_id, &system_address],
        ) {
            log::error!(
                "[Body][{}] deleting atmosphere_composition: {}",
                journal_id,
                err
            );
            return Err(EdcasError::from(err));
        }
        if let Some(atmosphere_composition) = atmosphere_composition {
            for atmosphere_composition in atmosphere_composition {
                atmosphere_composition.insert_into_db(
                    journal_id,
                    body_id,
                    system_address,
                    client,
                )?;
            }
        }
        if let Err(err) = client.execute(
            //language=postgresql
            "DELETE FROM planet_material WHERE body_id=$1 AND system_address=$2",
            &[&body_id, &system_address],
        ) {
            log::error!("[Body][{}] deleting planet_material: {}", journal_id, err);
            return Err(EdcasError::from(err));
        }
        if let Some(materials) = materials {
            for material in materials {
                material.insert_into_db(journal_id, body_id, system_address, client)?;
            }
        }
        if let Err(err) = client.execute(
            //language=postgresql
            "DELETE FROM planet_composition WHERE body_id=$1 AND system_address=$2",
            &[&body_id, &system_address],
        ) {
            log::error!(
                "[Body][{}] deleting planet_composition: {}",
                journal_id,
                err
            );
            return Err(EdcasError::from(err));
        }
        if let Some(composition) = composition {
            composition.insert_into_db(journal_id, body_id, system_address, client)?;
        }

        for parent in parents {
            parent.insert_into_db(journal_id, system_address, body_id, client)?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AtmosphereComposition {
    #[serde(rename = "Percent")]
    percent: f32,

    #[serde(rename = "Name")]
    name: String,
}
impl AtmosphereComposition {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        body_id: i32,
        system_address: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};

        let Self { percent, name } = self;
        let name = value_table(Tables::AtmosphereType, name, journal_id, client)?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO atmosphere_composition (atmosphere_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
            &[&name,&body_id,&system_address,&percent,&journal_id]
        ){
            log::error!("[AtmosphereComposition]: inserting atmosphere_composition: {}", err);
            return Err(crate::eddn::edcas_error::EdcasError::from(err));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Materials {
    #[serde(rename = "Percent")]
    percent: f32,

    #[serde(rename = "Name")]
    name: String,
}
impl Materials {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        body_id: i32,
        system_address: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};

        let Self { percent, name } = self;
        let name = value_table(Tables::MaterialType, name, journal_id, client)?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO atmosphere_composition (atmosphere_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
            &[&name,&body_id,&system_address,&percent,&journal_id]
        ){
            log::error!("[{}] inserting atmosphere_composition: {}",journal_id, err);
            return Err(crate::eddn::edcas_error::EdcasError::from(err));
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Composition {
    #[serde(rename = "Rock")]
    rock: f32,

    #[serde(rename = "Ice")]
    ice: f32,

    #[serde(rename = "Metal")]
    metal: f32,
}
impl Composition {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        body_id: i32,
        system_address: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::edcas::tables::{value_table, Tables};
        use crate::eddn::edcas_error::EdcasError;

        let Self { rock, ice, metal } = self;
        let composition_type = value_table(
            Tables::PlanetCompositionType,
            "Rock".to_string(),
            journal_id,
            client,
        )?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO planet_composition (composition_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
            &[&composition_type,&body_id,&system_address,&rock,&journal_id]
        ){

            log::error!("[{}] inserting planet_composition: {}",journal_id, err);
            return Err(EdcasError::from(err));
        }
        let composition_type = value_table(
            Tables::PlanetCompositionType,
            "Ice".to_string(),
            journal_id,
            client,
        )?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO planet_composition (composition_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
            &[&composition_type,&body_id,&system_address,&ice,&journal_id]
        ){

            log::error!("[{}] inserting planet_composition: {}",journal_id, err);
            return Err(EdcasError::from(err));
        }
        let composition_type = value_table(
            Tables::PlanetCompositionType,
            "Metal".to_string(),
            journal_id,
            client,
        )?;
        if let Err(err) = client.execute(
            //language=postgresql
            "INSERT INTO planet_composition (composition_type, body_id, system_address, percent, journal_id) VALUES ($1,$2,$3,$4,$5)",
            &[&composition_type,&body_id,&system_address,&metal,&journal_id]
        ){
            log::error!("[{}] inserting planet_composition: {}",journal_id, err);
            return Err(EdcasError::from(err));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct Parent {
    #[serde(rename = "Star")]
    star: Option<i32>,
    #[serde(rename = "Planet")]
    planet: Option<i32>,
    #[serde(rename = "Ring")]
    ring: Option<i32>,
    #[serde(rename = "Null")]
    null: Option<i32>,
}
impl Parent {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        body_id: i32,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::eddn::edcas_error::EdcasError;

        let Self {
            star,
            planet,
            ring,
            null,
        } = self;

        assert!(
            star.is_some() || planet.is_some() || ring.is_some() || null.is_some(),
            "No parent found"
        );

        if let Some(star) = star {
            if let Err(err) = client.execute(
                "INSERT INTO parents (type, parent_id, body_id, system_address, journal_id) VALUES ($1,$2,$3,$4,$5) ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                &[&"Star",&star,&body_id,&system_address,&journal_id]
            ){
                log::error!("[{}] inserting parents: {}",journal_id, err);
                return Err(EdcasError::from(err));
            }
        }
        if let Some(planet) = planet {
            if let Err(err) = client.execute(
                "INSERT INTO parents (type, parent_id, body_id, system_address, journal_id) VALUES ($1,$2,$3,$4,$5) ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                &[&"Planet",&planet,&body_id,&system_address,&journal_id]
            ){
                log::error!("[{}] inserting parents: {}",journal_id, err);
                return Err(EdcasError::from(err));
            }
        }
        if let Some(ring) = ring {
            if let Err(err) = client.execute(
                "INSERT INTO parents (type, parent_id, body_id, system_address, journal_id) VALUES ($1,$2,$3,$4,$5) ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                &[&"Ring",&ring,&body_id,&system_address,&journal_id]
            ){
                log::error!("[{}] inserting parents: {}",journal_id, err);
                return Err(EdcasError::from(err));
            }
        }
        if let Some(null) = null {
            if let Err(err) = client.execute(
                "INSERT INTO parents (type, parent_id, body_id, system_address, journal_id) VALUES ($1,$2,$3,$4,$5) ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                &[&"Null",&null,&body_id,&system_address,&journal_id]
            ){
                log::error!("[{}] inserting parents: {}",journal_id, err);
                return Err(EdcasError::from(err));
            }
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
pub struct Ring {
    #[serde(rename = "RingClass")]
    ring_class: String,

    #[serde(rename = "InnerRad")]
    inner_rad: f32,

    #[serde(rename = "OuterRad")]
    outer_rad: f32,

    #[serde(rename = "MassMT")]
    mass_mt: f32,

    #[serde(rename = "Name")]
    name: String,
}
impl Ring {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        system_address: i64,
        body_id: i32,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::{edcas::tables::value_table, eddn::edcas_error::EdcasError};
        let Self {
            ring_class,
            inner_rad,
            outer_rad,
            mass_mt,
            name,
        } = self;
        let ring_class = value_table(
            crate::edcas::tables::Tables::RingClass,
            ring_class,
            journal_id,
            client,
        )?;

        if let Err(err) = client.execute(
            "INSERT INTO ring (body_id,system_address,ring_class,inner_rad,outer_rad,mass_mt,name,journal_id) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT ON CONSTRAINT ring_pkey do update set ring_class=$3,inner_rad=$4,outer_rad=$5,mass_mt=$6,name=$7,journal_id=$8",
            &[&body_id,&system_address,&ring_class,&inner_rad,&outer_rad,&mass_mt,&name,&journal_id]
        ){
            log::error!("[Ring] inserting ring: {}", err);
            return Err(EdcasError::from(err));
        }
        Ok(())
    }
}
