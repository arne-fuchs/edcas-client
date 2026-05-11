use deadpool_postgres::Pool;
use edcas_common::journal::Scan;

use super::tables::lookup_or_insert;

pub async fn insert_scan(pool: &Pool, journal_id: i64, event: &Scan) -> anyhow::Result<()> {
    if event.is_star() {
        insert_star(pool, journal_id, event).await
    } else {
        insert_body(pool, journal_id, event).await
    }
}

async fn insert_star(pool: &Pool, journal_id: i64, event: &Scan) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    let star_type = lookup_or_insert(
        &tx,
        "star_type",
        event.star_type.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;

    tx.execute(
        "INSERT INTO star
            (id, system_address, name, stellar_mass, radius, surface_temperature,
             star_type, luminosity, age_my, journal_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
         ON CONFLICT ON CONSTRAINT star_pkey DO UPDATE SET
            stellar_mass=$3, radius=$4, surface_temperature=$5,
            star_type=$7, luminosity=$8, age_my=$9, journal_id=$10",
        &[
            &event.body_id,
            &event.system_address,
            &event.body_name,
            &event.stellar_mass,
            &event.radius,
            &event.surface_temperature,
            &star_type,
            &event.luminosity,
            &event.age_my,
            &journal_id,
        ],
    )
    .await?;

    if let Some(ref rings) = event.rings {
        for ring in rings {
            let ring_class =
                lookup_or_insert(&tx, "ring_class", &ring.ring_class, journal_id).await?;
            tx.execute(
                "INSERT INTO ring
                    (body_id, system_address, ring_class, inner_rad, outer_rad, mass_mt, name, journal_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                 ON CONFLICT ON CONSTRAINT ring_pkey DO UPDATE SET
                    ring_class=$3, inner_rad=$4, outer_rad=$5, mass_mt=$6, name=$7, journal_id=$8",
                &[
                    &event.body_id,
                    &event.system_address,
                    &ring_class,
                    &(ring.inner_rad as f32),
                    &(ring.outer_rad as f32),
                    &(ring.mass_mt as f32),
                    &ring.name,
                    &journal_id,
                ],
            )
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

async fn insert_body(pool: &Pool, journal_id: i64, event: &Scan) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    let planet_class = lookup_or_insert(
        &tx,
        "planet_class",
        event.planet_class.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;
    let volcanism = lookup_or_insert(
        &tx,
        "volcanism",
        event.volcanism.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;
    let atmosphere = lookup_or_insert(
        &tx,
        "atmosphere",
        event.atmosphere.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;
    let atmosphere_type = lookup_or_insert(
        &tx,
        "atmosphere_type",
        event.atmosphere_type.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;
    let terraform_state = lookup_or_insert(
        &tx,
        "terraform_state",
        event.terraform_state.as_deref().unwrap_or(""),
        journal_id,
    )
    .await?;

    tx.execute(
        "INSERT INTO body
            (id, system_address, name, mass_em, radius, landable, axial_tilt, periapsis,
             tidal_lock, volcanism, mapped, atmosphere, mean_anomaly, planet_class,
             eccentricity, ascending_node, orbital_period, semi_major_axis, atmosphere_type,
             rotation_period, surface_gravity, terraform_state, surface_pressure,
             orbital_inclination, surface_temperature, distance, journal_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26,$27)
         ON CONFLICT ON CONSTRAINT body_pkey DO UPDATE SET
            mass_em=$4, radius=$5, landable=$6, axial_tilt=$7, periapsis=$8, tidal_lock=$9,
            volcanism=$10, mapped=$11, atmosphere=$12, mean_anomaly=$13, planet_class=$14,
            eccentricity=$15, ascending_node=$16, orbital_period=$17, semi_major_axis=$18,
            atmosphere_type=$19, rotation_period=$20, surface_gravity=$21, terraform_state=$22,
            surface_pressure=$23, orbital_inclination=$24, surface_temperature=$25,
            distance=$26, journal_id=$27",
        &[
            &event.body_id,
            &event.system_address,
            &event.body_name,
            &event.mass_em,
            &event.radius,
            &event.landable,
            &event.axial_tilt,
            &event.periapsis,
            &event.tidal_lock,
            &volcanism,
            &event.was_mapped,
            &atmosphere,
            &event.mean_anomaly,
            &planet_class,
            &event.eccentricity,
            &event.ascending_node,
            &event.orbital_period,
            &event.semi_major_axis,
            &atmosphere_type,
            &event.rotation_period,
            &event.surface_gravity,
            &terraform_state,
            &event.surface_pressure,
            &event.orbital_inclination,
            &event.surface_temperature,
            &event.distance_from_arrival_ls,
            &journal_id,
        ],
    )
    .await?;

    // Atmosphere composition
    tx.execute(
        "DELETE FROM atmosphere_composition WHERE body_id=$1 AND system_address=$2",
        &[&event.body_id, &event.system_address],
    )
    .await?;
    if let Some(ref comps) = event.atmosphere_composition {
        for comp in comps {
            let atm_type =
                lookup_or_insert(&tx, "atmosphere_type", &comp.name, journal_id).await?;
            tx.execute(
                "INSERT INTO atmosphere_composition
                    (atmosphere_type, body_id, system_address, percent, journal_id)
                 VALUES ($1,$2,$3,$4,$5)",
                &[&atm_type, &event.body_id, &event.system_address, &comp.percent, &journal_id],
            )
            .await?;
        }
    }

    // Materials
    tx.execute(
        "DELETE FROM planet_material WHERE body_id=$1 AND system_address=$2",
        &[&event.body_id, &event.system_address],
    )
    .await?;
    if let Some(ref mats) = event.materials {
        for mat in mats {
            let mat_type =
                lookup_or_insert(&tx, "material_type", &mat.name, journal_id).await?;
            tx.execute(
                "INSERT INTO planet_material
                    (material_type, body_id, system_address, percent, journal_id)
                 VALUES ($1,$2,$3,$4,$5)",
                &[&mat_type, &event.body_id, &event.system_address, &(mat.percent as f32), &journal_id],
            )
            .await?;
        }
    }

    // Composition
    tx.execute(
        "DELETE FROM planet_composition WHERE body_id=$1 AND system_address=$2",
        &[&event.body_id, &event.system_address],
    )
    .await?;
    if let Some(ref comp) = event.composition {
        for (comp_name, percent) in [("Rock", comp.rock), ("Ice", comp.ice), ("Metal", comp.metal)] {
            let comp_type =
                lookup_or_insert(&tx, "planet_composition_type", comp_name, journal_id).await?;
            tx.execute(
                "INSERT INTO planet_composition
                    (composition_type, body_id, system_address, percent, journal_id)
                 VALUES ($1,$2,$3,$4,$5)",
                &[&comp_type, &event.body_id, &event.system_address, &percent, &journal_id],
            )
            .await?;
        }
    }

    // Rings
    if let Some(ref rings) = event.rings {
        for ring in rings {
            let ring_class =
                lookup_or_insert(&tx, "ring_class", &ring.ring_class, journal_id).await?;
            tx.execute(
                "INSERT INTO ring
                    (body_id, system_address, ring_class, inner_rad, outer_rad, mass_mt, name, journal_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
                 ON CONFLICT ON CONSTRAINT ring_pkey DO UPDATE SET
                    ring_class=$3, inner_rad=$4, outer_rad=$5, mass_mt=$6, name=$7, journal_id=$8",
                &[
                    &event.body_id,
                    &event.system_address,
                    &ring_class,
                    &(ring.inner_rad as f32),
                    &(ring.outer_rad as f32),
                    &(ring.mass_mt as f32),
                    &ring.name,
                    &journal_id,
                ],
            )
            .await?;
        }
    }

    // Parents
    if let Some(ref parents) = event.parents {
        for parent in parents {
            if let Some(pid) = parent.parent_id() {
                tx.execute(
                    "INSERT INTO parents
                        (type, parent_id, body_id, system_address, journal_id)
                     VALUES ($1,$2,$3,$4,$5)
                     ON CONFLICT ON CONSTRAINT parents_pkey DO NOTHING",
                    &[
                        &parent.parent_type(),
                        &pid,
                        &event.body_id,
                        &event.system_address,
                        &journal_id,
                    ],
                )
                .await?;
            }
        }
    }

    tx.commit().await?;
    Ok(())
}
