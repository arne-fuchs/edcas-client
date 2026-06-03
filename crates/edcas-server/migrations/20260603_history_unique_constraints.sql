-- Make the append-only history tables idempotent.
--
-- The edcas client now also uploads to the EDDN network. Because this server already
-- ingests the global EDDN firehose, a client's data can reach the database twice: once
-- directly via the REST API and once via the EDDN relay. Domain tables already upsert
-- idempotently, but these history tables were plain INSERTs. Adding unique constraints
-- (combined with `ON CONFLICT DO NOTHING` in the inserters) makes ingestion idempotent
-- regardless of how many times the same snapshot arrives.
--
-- Existing duplicate rows are removed before the constraints are added. The dedup steps
-- are no-ops on fresh/empty tables, so this migration is safe for new installs too.

DELETE FROM commodity_price_history a
USING commodity_price_history b
WHERE a.id > b.id
  AND a.market_id = b.market_id
  AND a.name = b.name
  AND a.event_timestamp = b.event_timestamp;

ALTER TABLE commodity_price_history
    ADD CONSTRAINT commodity_price_history_unique
    UNIQUE (market_id, name, event_timestamp);

DELETE FROM system_population_history a
USING system_population_history b
WHERE a.id > b.id
  AND a.system_address = b.system_address
  AND a.event_timestamp = b.event_timestamp;

ALTER TABLE system_population_history
    ADD CONSTRAINT system_population_history_unique
    UNIQUE (system_address, event_timestamp);

DELETE FROM faction_influence_history a
USING faction_influence_history b
WHERE a.id > b.id
  AND a.faction_name = b.faction_name
  AND a.system_address = b.system_address
  AND a.event_timestamp = b.event_timestamp;

ALTER TABLE faction_influence_history
    ADD CONSTRAINT faction_influence_history_unique
    UNIQUE (faction_name, system_address, event_timestamp);
