use deadpool_postgres::Pool;
use edcas_common::journal::{CarrierJump, FsdJump, Location};

use super::tables::lookup_or_insert;

pub async fn insert_fsd_jump(pool: &Pool, journal_id: i64, event: &FsdJump) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    let allegiance = lookup_or_insert(&tx, "allegiance", &event.system_allegiance, journal_id).await?;
    let economy = lookup_or_insert(&tx, "economy_type", &event.system_economy, journal_id).await?;
    let second_economy = lookup_or_insert(&tx, "economy_type", &event.system_second_economy, journal_id).await?;
    let government = lookup_or_insert(&tx, "government", &event.system_government, journal_id).await?;
    let security = lookup_or_insert(&tx, "security", &event.system_security, journal_id).await?;

    let controlling_power = if let Some(ref cp) = event.controlling_power {
        let id = lookup_or_insert(&tx, "power", cp, journal_id).await?;
        Some(id)
    } else {
        None
    };

    if let Some(ref powers) = event.powers {
        for power in powers {
            lookup_or_insert(&tx, "power", power, journal_id).await?;
        }
    }

    let x = event.star_pos.first().copied().unwrap_or(0.0);
    let y = event.star_pos.get(1).copied().unwrap_or(0.0);
    let z = event.star_pos.get(2).copied().unwrap_or(0.0);

    tx.execute(
        "INSERT INTO star_systems
            (system_address, name, x, y, z, allegiance, economy, second_economy, government,
             security, population, controlling_power, journal_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
         ON CONFLICT (system_address) DO UPDATE SET
            name=$2, x=$3, y=$4, z=$5, allegiance=$6, economy=$7, second_economy=$8,
            government=$9, security=$10, population=$11, controlling_power=$12, journal_id=$13",
        &[
            &event.system_address,
            &event.star_system,
            &x, &y, &z,
            &allegiance, &economy, &second_economy, &government, &security,
            &event.population, &controlling_power, &journal_id,
        ],
    )
    .await?;

    if let Some(ref factions) = event.factions {
        for faction in factions {
            insert_faction(&tx, journal_id, event.system_address, faction).await?;
        }
    }

    if let Some(ref conflicts) = event.conflicts {
        for conflict in conflicts {
            let war_type = lookup_or_insert(&tx, "war_type", &conflict.war_type, journal_id).await?;
            tx.execute(
                "INSERT INTO conflicts
                    (system_address, war_type, status, faction1_name, faction1_stake, faction1_won_days,
                     faction2_name, faction2_stake, faction2_won_days, journal_id)
                 VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                 ON CONFLICT DO NOTHING",
                &[
                    &event.system_address, &war_type, &conflict.status,
                    &conflict.faction1.name, &conflict.faction1.stake, &conflict.faction1.won_days,
                    &conflict.faction2.name, &conflict.faction2.stake, &conflict.faction2.won_days,
                    &journal_id,
                ],
            )
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert_location(pool: &Pool, journal_id: i64, event: &Location) -> anyhow::Result<()> {
    let fsdjump_like = FsdJump {
        timestamp: event.timestamp.clone(),
        star_system: event.star_system.clone(),
        system_address: event.system_address,
        star_pos: event.star_pos.clone(),
        body: event.body.clone(),
        body_id: event.body_id as i32,
        body_type: event.body_type.clone(),
        population: event.population,
        system_economy: event.system_economy.clone(),
        system_second_economy: event.system_second_economy.clone(),
        system_government: event.system_government.clone(),
        system_allegiance: event.system_allegiance.clone(),
        system_security: event.system_security.clone(),
        factions: event.factions.clone(),
        system_faction: event.system_faction.clone(),
        conflicts: event.conflicts.clone(),
        controlling_power: event.controlling_power.clone(),
        powers: event.powers.clone(),
        multicrew: event.multicrew,
        jump_dist: None,
        fuel_used: None,
        fuel_level: None,
        horizons: event.horizons,
        odyssey: event.odyssey,
    };
    insert_fsd_jump(pool, journal_id, &fsdjump_like).await?;

    if event.docked {
        if let (Some(market_id), Some(station_name), Some(station_type)) =
            (event.market_id, &event.station_name, &event.station_type)
        {
            let docked = edcas_common::journal::Docked {
                timestamp: event.timestamp.clone(),
                station_name: station_name.clone(),
                station_type: station_type.clone(),
                market_id,
                system_address: event.system_address,
                star_system: event.star_system.clone(),
                station_faction: event.station_faction.clone(),
                station_government: event.station_government.clone().unwrap_or_default(),
                station_allegiance: event.station_allegiance.clone().unwrap_or_default(),
                station_services: event.station_services.clone(),
                station_economy: event.station_economy.clone().unwrap_or_default(),
                station_economies: event.station_economies.clone(),
                landing_pads: None,
                dist_from_star_ls: event.dist_from_star_ls,
                wanted: false,
                active_fine: false,
                taxi: event.taxi,
                horizons: event.horizons,
                odyssey: event.odyssey,
            };
            super::station::insert_docked(pool, journal_id, &docked).await?;
        }
    }

    Ok(())
}

pub async fn insert_carrier_jump(pool: &Pool, journal_id: i64, event: &CarrierJump) -> anyhow::Result<()> {
    let fsdjump_like = FsdJump {
        timestamp: event.timestamp.clone(),
        star_system: event.star_system.clone(),
        system_address: event.system_address,
        star_pos: event.star_pos.clone(),
        body: event.body.clone(),
        body_id: event.body_id,
        body_type: event.body_type.clone(),
        population: event.population,
        system_economy: event.system_economy.clone(),
        system_second_economy: event.system_second_economy.clone(),
        system_government: event.system_government.clone(),
        system_allegiance: event.system_allegiance.clone(),
        system_security: event.system_security.clone(),
        factions: event.factions.clone(),
        system_faction: event.system_faction.clone(),
        conflicts: event.conflicts.clone(),
        controlling_power: event.controlling_power.clone(),
        powers: event.powers.clone(),
        multicrew: false,
        jump_dist: None,
        fuel_used: None,
        fuel_level: None,
        horizons: event.horizons,
        odyssey: event.odyssey,
    };
    insert_fsd_jump(pool, journal_id, &fsdjump_like).await?;

    // Insert station record for the carrier itself
    if let (Some(market_id), Some(station_name), Some(station_type)) =
        (event.market_id, &event.station_name, &event.station_type)
    {
        let docked = edcas_common::journal::Docked {
            timestamp: event.timestamp.clone(),
            station_name: station_name.clone(),
            station_type: station_type.clone(),
            market_id,
            system_address: event.system_address,
            star_system: event.star_system.clone(),
            station_faction: event.station_faction.clone(),
            station_government: String::new(),
            station_allegiance: String::new(),
            station_services: event.station_services.clone(),
            station_economy: event.station_economy.clone().unwrap_or_default(),
            station_economies: event.station_economies.clone(),
            landing_pads: None,
            dist_from_star_ls: None,
            wanted: false,
            active_fine: false,
            taxi: false,
            horizons: event.horizons,
            odyssey: event.odyssey,
        };
        super::station::insert_docked(pool, journal_id, &docked).await?;
    }

    Ok(())
}

async fn insert_faction(
    tx: &tokio_postgres::Transaction<'_>,
    journal_id: i64,
    system_address: i64,
    faction: &edcas_common::journal::types::Faction,
) -> anyhow::Result<()> {
    let government = lookup_or_insert(tx, "government", &faction.government, journal_id).await?;
    let allegiance = lookup_or_insert(tx, "allegiance", &faction.allegiance, journal_id).await?;
    let happiness = lookup_or_insert(tx, "happiness", &faction.happiness, journal_id).await?;

    tx.execute(
        "INSERT INTO factions
            (name, system_address, government, allegiance, happiness, influence, journal_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT ON CONSTRAINT factions_pkey DO UPDATE SET
            government=$3, allegiance=$4, happiness=$5, influence=$6, journal_id=$7",
        &[
            &faction.name,
            &system_address,
            &government,
            &allegiance,
            &happiness,
            &faction.influence,
            &journal_id,
        ],
    )
    .await?;

    if let Some(ref active) = faction.active_states {
        for state in active {
            let state_id = lookup_or_insert(tx, "faction_state_name", &state.state, journal_id).await?;
            tx.execute(
                "INSERT INTO faction_states (faction_name, system_address, state, status, journal_id)
                 VALUES ($1,$2,$3,'Active',$4) ON CONFLICT DO NOTHING",
                &[&faction.name, &system_address, &state_id, &journal_id],
            )
            .await?;
        }
    }

    Ok(())
}
