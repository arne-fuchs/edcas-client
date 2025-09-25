use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Scan {
    #[serde(rename = "SystemAddress")]
    system_address: i64,

    #[serde(rename = "Radius")]
    radius: f32,

    #[serde(rename = "AxialTilt")]
    axial_tilt: f32,

    #[serde(rename = "OrbitalPeriod")]
    orbital_period: f32,

    #[serde(rename = "Eccentricity")]
    eccentricity: f32,

    #[serde(rename = "Parents")]
    parents: Vec<crate::edcas::assets::body::Parent>,

    #[serde(rename = "SurfacePressure")]
    surface_pressure: i32,

    #[serde(rename = "OrbitalInclination")]
    orbital_inclination: f32,

    #[serde(rename = "RotationPeriod")]
    rotation_period: f32,

    #[serde(rename = "MassEM")]
    mass_em: f32,

    #[serde(rename = "TerraformState")]
    terraform_state: String,

    #[serde(rename = "DistanceFromArrivalLS")]
    distance_from_arrival_ls: f32,

    #[serde(rename = "odyssey")]
    odyssey: bool,

    #[serde(rename = "SemiMajorAxis")]
    semi_major_axis: f32,

    #[serde(rename = "SurfaceGravity")]
    surface_gravity: f32,

    #[serde(rename = "SurfaceTemperature")]
    surface_temperature: f32,

    #[serde(rename = "Rings")]
    rings: Vec<crate::edcas::assets::body::Ring>,

    #[serde(rename = "AscendingNode")]
    ascending_node: f32,

    #[serde(rename = "AtmosphereComposition")]
    atmosphere_composition: Vec<crate::edcas::assets::body::AtmosphereComposition>,

    #[serde(rename = "event")]
    event: String,

    #[serde(rename = "Landable")]
    landable: bool,

    #[serde(rename = "Volcanism")]
    volcanism: String,

    #[serde(rename = "WasDiscovered")]
    was_discovered: bool,

    #[serde(rename = "timestamp")]
    timestamp: String,

    #[serde(rename = "StarPos")]
    star_pos: Vec<f32>,

    #[serde(rename = "Composition")]
    composition: crate::edcas::assets::body::Composition,

    #[serde(rename = "AtmosphereType")]
    atmosphere_type: String,

    #[serde(rename = "StarSystem")]
    star_system: String,

    #[serde(rename = "WasMapped")]
    was_mapped: bool,

    #[serde(rename = "PlanetClass")]
    planet_class: String,

    #[serde(rename = "horizons")]
    horizons: bool,

    #[serde(rename = "ScanType")]
    scan_type: String,

    #[serde(rename = "MeanAnomaly")]
    mean_anomaly: f32,

    #[serde(rename = "ReserveLevel")]
    reserve_level: String,

    #[serde(rename = "BodyID")]
    body_id: i32,

    #[serde(rename = "Atmosphere")]
    atmosphere: String,

    #[serde(rename = "Periapsis")]
    periapsis: f32,

    #[serde(rename = "BodyName")]
    body_name: String,

    #[serde(rename = "TidalLock")]
    tidal_lock: bool,
}
impl Scan {
    #[cfg(feature = "eddn")]
    pub fn insert_into_db(
        self,
        journal_id: i64,
        client: &mut postgres::Client,
    ) -> Result<(), crate::eddn::edcas_error::EdcasError> {
        use crate::{
            edcas::tables::value_table, edcas::tables::Tables, eddn::edcas_error::EdcasError,
        };
        let Self {
            system_address,
            radius,
            axial_tilt,
            orbital_period,
            eccentricity,
            parents,
            surface_pressure,
            orbital_inclination,
            rotation_period,
            mass_em,
            terraform_state,
            distance_from_arrival_ls,
            odyssey,
            semi_major_axis,
            surface_gravity,
            surface_temperature,
            rings,
            ascending_node,
            atmosphere_composition,
            event,
            landable,
            volcanism,
            was_discovered,
            timestamp,
            star_pos,
            composition,
            atmosphere_type,
            star_system,
            was_mapped,
            planet_class,
            horizons,
            scan_type,
            mean_anomaly,
            reserve_level,
            body_id,
            atmosphere,
            periapsis,
            body_name,
            tidal_lock,
        } = self;

        let planet_class = value_table(Tables::PlanetClass, planet_class, journal_id, client)?;
        let volcanism = value_table(Tables::Volcanism, volcanism, journal_id, client)?;
        let atmosphere = value_table(Tables::Atmosphere, atmosphere, journal_id, client)?;
        let atmosphere_type =
            value_table(Tables::AtmosphereType, atmosphere_type, journal_id, client)?;
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
                            log::error!("[Body] inserting body: {}", err);
                            return Err(EdcasError::from(err));
                        }
        Ok(())
    }
}
