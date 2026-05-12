use chrono::{DateTime, Utc};
use deadpool_postgres::Pool;
use edcas_common::journal::station::{Commodities, Docked, Outfitting, Shipyard};

use super::tables::lookup_or_insert;

pub async fn insert_docked(
    pool: &Pool,
    journal_id: i64,
    event_timestamp: DateTime<Utc>,
    event: &Docked,
) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    // Serialise concurrent writes for the same market to prevent deadlocks.
    tx.execute("SELECT pg_advisory_xact_lock($1)", &[&event.market_id]).await?;

    let station_type =
        lookup_or_insert(&tx, "station_type", &event.station_type, journal_id).await?;
    let government =
        lookup_or_insert(&tx, "government", &event.station_government, journal_id).await?;
    let economy =
        lookup_or_insert(&tx, "economy_type", &event.station_economy, journal_id).await?;

    let faction_name = event
        .station_faction
        .as_ref()
        .map(|f| f.name.as_str())
        .unwrap_or("");

    tx.execute(
        "INSERT INTO stations
            (market_id, system_address, name, type, faction_name, government, economy,
             journal_id, event_timestamp)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
         ON CONFLICT ON CONSTRAINT stations_pkey DO UPDATE SET
            system_address=$2, name=$3, type=$4, faction_name=$5,
            government=$6, economy=$7, journal_id=$8, event_timestamp=$9
         WHERE EXCLUDED.event_timestamp >= stations.event_timestamp",
        &[
            &event.market_id,
            &event.system_address,
            &event.station_name,
            &station_type,
            &faction_name,
            &government,
            &economy,
            &journal_id,
            &event_timestamp,
        ],
    )
    .await?;

    // Station services
    tx.execute(
        "DELETE FROM station_services WHERE market_id=$1",
        &[&event.market_id],
    )
    .await?;
    if let Some(ref services) = event.station_services {
        for service in services {
            let svc_id =
                lookup_or_insert(&tx, "station_services_types", service, journal_id).await?;
            tx.execute(
                "INSERT INTO station_services (id, market_id, journal_id) VALUES ($1,$2,$3)",
                &[&svc_id, &event.market_id, &journal_id],
            )
            .await?;
        }
    }

    // Station economies
    if let Some(ref economies) = event.station_economies {
        for econ in economies {
            let econ_id =
                lookup_or_insert(&tx, "economy_type", &econ.name, journal_id).await?;
            tx.execute(
                "INSERT INTO station_economies (market_id, economy_type, proportion, journal_id)
                 VALUES ($1,$2,$3,$4)
                 ON CONFLICT DO NOTHING",
                &[&event.market_id, &econ_id, &econ.proportion, &journal_id],
            )
            .await?;
        }
    }

    // Landing pads
    if let Some(ref pads) = event.landing_pads {
        tx.execute(
            "INSERT INTO station_landing_pads (market_id, small, medium, large, journal_id)
             VALUES ($1,$2,$3,$4,$5)
             ON CONFLICT (market_id) DO UPDATE SET small=$2, medium=$3, large=$4, journal_id=$5",
            &[&event.market_id, &pads.small, &pads.medium, &pads.large, &journal_id],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert_commodities(
    pool: &Pool,
    journal_id: i64,
    event_timestamp: DateTime<Utc>,
    event: &Commodities,
) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    tx.execute("SELECT pg_advisory_xact_lock($1)", &[&event.market_id]).await?;

    // Skip entirely if existing data is newer.
    if let Some(row) = tx
        .query_opt(
            "SELECT event_timestamp FROM commodity_listening WHERE market_id=$1 LIMIT 1",
            &[&event.market_id],
        )
        .await?
    {
        let existing: DateTime<Utc> = row.get(0);
        if existing > event_timestamp {
            tx.commit().await?;
            return Ok(());
        }
    }

    tx.execute(
        "DELETE FROM commodity_listening WHERE market_id=$1",
        &[&event.market_id],
    )
    .await?;

    for commodity in &event.commodities {
        tx.execute(
            "INSERT INTO commodity_listening
                (market_id, name, mean_price, buy_price, stock, stock_bracket,
                 sell_price, demand, demand_bracket, journal_id, event_timestamp)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
             ON CONFLICT (market_id, name) DO UPDATE SET
                mean_price    = EXCLUDED.mean_price,
                buy_price     = EXCLUDED.buy_price,
                stock         = EXCLUDED.stock,
                stock_bracket = EXCLUDED.stock_bracket,
                sell_price    = EXCLUDED.sell_price,
                demand        = EXCLUDED.demand,
                demand_bracket = EXCLUDED.demand_bracket,
                journal_id    = EXCLUDED.journal_id,
                event_timestamp = EXCLUDED.event_timestamp",
            &[
                &event.market_id,
                &commodity.name,
                &commodity.mean_price,
                &commodity.buy_price,
                &commodity.stock,
                &commodity.stock_bracket,
                &commodity.sell_price,
                &commodity.demand,
                &commodity.demand_bracket,
                &journal_id,
                &event_timestamp,
            ],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert_outfitting(
    pool: &Pool,
    journal_id: i64,
    event_timestamp: DateTime<Utc>,
    event: &Outfitting,
) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    tx.execute("SELECT pg_advisory_xact_lock($1)", &[&event.market_id]).await?;

    // Skip entirely if existing data is newer.
    if let Some(row) = tx
        .query_opt(
            "SELECT event_timestamp FROM modul_listening WHERE market_id=$1 LIMIT 1",
            &[&event.market_id],
        )
        .await?
    {
        let existing: DateTime<Utc> = row.get(0);
        if existing > event_timestamp {
            tx.commit().await?;
            return Ok(());
        }
    }

    tx.execute(
        "DELETE FROM modul_listening WHERE market_id=$1",
        &[&event.market_id],
    )
    .await?;

    for module in &event.modules {
        tx.execute(
            "INSERT INTO modul_listening (market_id, id, category, name, cost, ship, journal_id, event_timestamp)
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
             ON CONFLICT (market_id, id) DO UPDATE SET
                category   = EXCLUDED.category,
                name       = EXCLUDED.name,
                cost       = EXCLUDED.cost,
                ship       = EXCLUDED.ship,
                journal_id = EXCLUDED.journal_id,
                event_timestamp = EXCLUDED.event_timestamp",
            &[
                &event.market_id,
                &module.id,
                &module.category,
                &module.name,
                &module.cost,
                &module.ship,
                &journal_id,
                &event_timestamp,
            ],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert_shipyard(
    pool: &Pool,
    journal_id: i64,
    event_timestamp: DateTime<Utc>,
    event: &Shipyard,
) -> anyhow::Result<()> {
    let mut client = pool.get().await?;
    let tx = client.build_transaction().start().await?;

    tx.execute("SELECT pg_advisory_xact_lock($1)", &[&event.market_id]).await?;

    // Skip entirely if existing data is newer.
    if let Some(row) = tx
        .query_opt(
            "SELECT event_timestamp FROM ship_listening WHERE market_id=$1 LIMIT 1",
            &[&event.market_id],
        )
        .await?
    {
        let existing: DateTime<Utc> = row.get(0);
        if existing > event_timestamp {
            tx.commit().await?;
            return Ok(());
        }
    }

    tx.execute(
        "DELETE FROM ship_listening WHERE market_id=$1",
        &[&event.market_id],
    )
    .await?;

    for ship in &event.ships {
        tx.execute(
            "INSERT INTO ship_listening (market_id, id, name, basevalue, journal_id, event_timestamp)
             VALUES ($1,$2,$3,$4,$5,$6)
             ON CONFLICT (market_id, id) DO UPDATE SET
                name       = EXCLUDED.name,
                basevalue  = EXCLUDED.basevalue,
                journal_id = EXCLUDED.journal_id,
                event_timestamp = EXCLUDED.event_timestamp",
            &[
                &event.market_id,
                &ship.id,
                &ship.name,
                &ship.base_value,
                &journal_id,
                &event_timestamp,
            ],
        )
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
