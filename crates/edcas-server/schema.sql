-- ============================================================
-- EDCAS-EDDN canonical schema
-- Matches what crates/edcas-eddn writes exactly.
-- Apply to a fresh database with: psql -f schema.sql
-- ============================================================

-- ── String-enum lookup tables ────────────────────────────────
-- Each stores unique string values referenced by integer ID.
-- journal_id links each first-seen value back to the event
-- that introduced it for auditability.

CREATE TABLE allegiance (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE power (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE economy_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE government (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE security (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE happiness (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE war_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE faction_state_name (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE station_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE station_services_types (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE star_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE planet_class (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE volcanism (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE atmosphere (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE atmosphere_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE terraform_state (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE ring_class (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE material_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

CREATE TABLE planet_composition_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

-- Signal types as reported in FSSBodySignals / SAASignalsFound
-- (e.g. "$SAA_SignalType_Biological;", "$SAA_SignalType_Geological;")
CREATE TABLE signal_type (
    id        SERIAL PRIMARY KEY,
    value     VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT NOT NULL
);

-- ── Raw event log ────────────────────────────────────────────
-- Every EDDN message is stored here before being dispatched to
-- typed tables. schema_ref is the EDDN schema URL.

CREATE TABLE journal_events (
    id              BIGSERIAL PRIMARY KEY,
    timestamp       TIMESTAMPTZ NOT NULL,
    event_timestamp TIMESTAMPTZ,
    event_type      VARCHAR(255) NOT NULL,
    schema_ref      VARCHAR(512),
    data            JSONB NOT NULL
);

-- ── Star systems ─────────────────────────────────────────────

CREATE TABLE star_systems (
    system_address  BIGINT PRIMARY KEY,
    name            VARCHAR(255) NOT NULL,
    x               REAL,
    y               REAL,
    z               REAL,
    allegiance      INTEGER REFERENCES allegiance(id),
    economy         INTEGER REFERENCES economy_type(id),
    second_economy  INTEGER REFERENCES economy_type(id),
    government      INTEGER REFERENCES government(id),
    security        INTEGER REFERENCES security(id),
    population      BIGINT,
    controlling_power INTEGER REFERENCES power(id),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

-- ── Factions ─────────────────────────────────────────────────
-- Faction names are stored directly as VARCHAR — the faction
-- name is the natural key and normalising it into a lookup
-- table adds indirection without benefit.

CREATE TABLE factions (
    name            VARCHAR(255) NOT NULL,
    system_address  BIGINT,
    government      INTEGER REFERENCES government(id),
    allegiance      INTEGER REFERENCES allegiance(id),
    happiness       INTEGER REFERENCES happiness(id),
    influence       REAL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (name, system_address)
);

-- Active BGS states for a faction in a system.
-- status is currently always 'Active' (pending/recovering handled
-- separately if needed in future).
CREATE TABLE faction_states (
    faction_name    VARCHAR(255) NOT NULL,
    system_address  BIGINT NOT NULL,
    state           INTEGER REFERENCES faction_state_name(id),
    status          VARCHAR(50) NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (faction_name, system_address, state),
    FOREIGN KEY (faction_name, system_address) REFERENCES factions(name, system_address)
);

-- System conflicts (one row per conflict, faction data inlined).
CREATE TABLE conflicts (
    id                  BIGSERIAL PRIMARY KEY,
    system_address      BIGINT,
    war_type            INTEGER REFERENCES war_type(id),
    status              VARCHAR(50),
    faction1_name       VARCHAR(255),
    faction1_stake      VARCHAR(255),
    faction1_won_days   INTEGER,
    faction2_name       VARCHAR(255),
    faction2_stake      VARCHAR(255),
    faction2_won_days   INTEGER,
    journal_id          BIGINT REFERENCES journal_events(id) NOT NULL
);

-- ── Stars ────────────────────────────────────────────────────

CREATE TABLE star (
    id                  INTEGER NOT NULL,
    system_address      BIGINT,
    name                VARCHAR(255) NOT NULL,
    stellar_mass        REAL,
    radius              REAL,
    surface_temperature REAL,
    star_type           INTEGER REFERENCES star_type(id),
    luminosity          VARCHAR(10),
    age_my              INTEGER,
    journal_id          BIGINT REFERENCES journal_events(id),
    event_timestamp     TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (id, system_address)
);

-- ── Planetary bodies ─────────────────────────────────────────

CREATE TABLE body (
    id                  INTEGER NOT NULL,
    system_address      BIGINT,
    name                VARCHAR(255) NOT NULL,
    mass_em             REAL,
    radius              REAL,
    landable            BOOL,
    axial_tilt          REAL,
    periapsis           REAL,
    tidal_lock          BOOL,
    volcanism           INTEGER REFERENCES volcanism(id),
    mapped              BOOL,
    atmosphere          INTEGER REFERENCES atmosphere(id),
    mean_anomaly        REAL,
    planet_class        INTEGER REFERENCES planet_class(id),
    eccentricity        REAL,
    ascending_node      REAL,
    orbital_period      REAL,
    semi_major_axis     REAL,
    atmosphere_type     INTEGER REFERENCES atmosphere_type(id),
    rotation_period     REAL,
    surface_gravity     REAL,
    terraform_state     INTEGER REFERENCES terraform_state(id),
    surface_pressure    REAL,
    orbital_inclination REAL,
    surface_temperature REAL,
    distance            REAL,
    journal_id          BIGINT REFERENCES journal_events(id),
    event_timestamp     TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (id, system_address)
);

CREATE TABLE ring (
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    name            VARCHAR(255) NOT NULL,
    ring_class      INTEGER REFERENCES ring_class(id),
    inner_rad       REAL,
    outer_rad       REAL,
    mass_mt         REAL,
    journal_id      BIGINT REFERENCES journal_events(id),
    PRIMARY KEY (body_id, system_address, name)
);

CREATE TABLE parents (
    type            VARCHAR(20),
    parent_id       INTEGER,
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id),
    PRIMARY KEY (parent_id, body_id, system_address)
);

CREATE TABLE atmosphere_composition (
    atmosphere_type INTEGER REFERENCES atmosphere_type(id),
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    percent         REAL NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_material (
    material_type   INTEGER REFERENCES material_type(id),
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    percent         REAL NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_composition (
    composition_type INTEGER REFERENCES planet_composition_type(id),
    body_id          INTEGER NOT NULL,
    system_address   BIGINT NOT NULL,
    percent          REAL NOT NULL,
    journal_id       BIGINT REFERENCES journal_events(id) NOT NULL
);

-- ── FSS / SAA signal tables ──────────────────────────────────
-- No FK to body — signals arrive before the body scan in EDDN
-- message order, so the body row may not exist yet.

CREATE TABLE fss_body_signals (
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    signal_type     INTEGER REFERENCES signal_type(id) NOT NULL,
    count           INTEGER NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (body_id, system_address, signal_type)
);

CREATE TABLE saa_signals (
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    signal_type     INTEGER REFERENCES signal_type(id) NOT NULL,
    count           INTEGER NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (body_id, system_address, signal_type)
);

-- ── Stations ─────────────────────────────────────────────────

CREATE TABLE stations (
    market_id       BIGINT PRIMARY KEY,
    system_address  BIGINT,
    name            VARCHAR(255) NOT NULL,
    carrier_name    VARCHAR(255),
    type            INTEGER REFERENCES station_type(id),
    faction_name    VARCHAR(255),
    government      INTEGER REFERENCES government(id),
    economy         INTEGER REFERENCES economy_type(id),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

CREATE TABLE station_services (
    id          INTEGER REFERENCES station_services_types(id),
    market_id   BIGINT REFERENCES stations(market_id),
    journal_id  BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (id, market_id)
);

-- economy_type column (not 'id') to avoid ambiguity with the table's own PK
CREATE TABLE station_economies (
    market_id    BIGINT REFERENCES stations(market_id),
    economy_type INTEGER REFERENCES economy_type(id),
    proportion   REAL NOT NULL,
    journal_id   BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (market_id, economy_type)
);

-- Flat small/medium/large columns — there are always exactly three pad sizes
CREATE TABLE station_landing_pads (
    market_id   BIGINT PRIMARY KEY REFERENCES stations(market_id),
    small       INTEGER,
    medium      INTEGER,
    large       INTEGER,
    journal_id  BIGINT REFERENCES journal_events(id) NOT NULL
);

-- ── Market data ──────────────────────────────────────────────

-- Commodity market (Commodities EDDN schema)
-- name stored as VARCHAR — commodity name is stable and unique
CREATE TABLE commodity_listening (
    market_id       BIGINT,
    name            VARCHAR(255) NOT NULL,
    mean_price      INTEGER,
    buy_price       INTEGER,
    stock           INTEGER,
    stock_bracket   INTEGER,
    sell_price      INTEGER,
    demand          INTEGER,
    demand_bracket  INTEGER,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (market_id, name)
);

-- Outfitting (Outfitting EDDN schema)
-- id is the module's internal identifier string (e.g. "Hpt_Mining_Laser_Fixed_Medium")
CREATE TABLE modul_listening (
    market_id       BIGINT,
    id              VARCHAR(255) NOT NULL,
    category        VARCHAR(255),
    name            VARCHAR(255),
    cost            BIGINT,
    ship            VARCHAR(255),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (market_id, id)
);

-- Shipyard (Shipyard EDDN schema)
CREATE TABLE ship_listening (
    market_id       BIGINT,
    id              VARCHAR(255) NOT NULL,
    name            VARCHAR(255),
    basevalue       BIGINT,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z',
    PRIMARY KEY (market_id, id)
);

-- ── Indexes ──────────────────────────────────────────────────

-- ── Colonisation construction depots ───────────────────────

CREATE TABLE construction_depots (
    market_id               BIGINT PRIMARY KEY,
    system_address          BIGINT NOT NULL,
    station_name            VARCHAR(255) NOT NULL,
    progress                REAL NOT NULL DEFAULT 0,
    construction_complete   BOOLEAN NOT NULL DEFAULT FALSE,
    construction_failed     BOOLEAN NOT NULL DEFAULT FALSE,
    last_updated            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    journal_id              BIGINT REFERENCES journal_events(id)
);

CREATE TABLE construction_resources (
    market_id       BIGINT NOT NULL REFERENCES construction_depots(market_id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    display_name    VARCHAR(255) NOT NULL,
    required_amount INTEGER NOT NULL,
    provided_amount INTEGER NOT NULL DEFAULT 0,
    payment         BIGINT NOT NULL DEFAULT 0,
    PRIMARY KEY (market_id, name)
);

CREATE INDEX idx_journal_events_type             ON journal_events (event_type);
CREATE INDEX idx_star_systems_name               ON star_systems (name);
CREATE INDEX idx_body_system                     ON body (system_address);
CREATE INDEX idx_star_system                     ON star (system_address);
CREATE INDEX idx_ring_system                     ON ring (system_address);
CREATE INDEX idx_parents_system                  ON parents (system_address);
CREATE INDEX idx_planet_material_system          ON planet_material (system_address);
CREATE INDEX idx_factions_system                 ON factions (system_address);
CREATE INDEX idx_stations_name                   ON stations (name);
CREATE INDEX idx_stations_system                 ON stations (system_address);
CREATE INDEX idx_fss_body_signals_system         ON fss_body_signals (system_address);
CREATE INDEX idx_saa_signals_system              ON saa_signals (system_address);
CREATE INDEX idx_construction_depots_system      ON construction_depots (system_address);
CREATE INDEX idx_construction_depots_name        ON construction_depots (LOWER(station_name));
CREATE INDEX idx_star_systems_coords             ON star_systems (x, y, z);
CREATE INDEX idx_commodity_listening_name        ON commodity_listening (name);

-- ── Pre-computed trade cache ─────────────────────────────────
-- Refreshed in the background every 15 minutes.
-- pad_filter: 'any' = no filter, 'M' = medium+ pads, 'L' = large pads only.

CREATE TABLE cached_trade_routes (
    pad_filter          VARCHAR(3)   NOT NULL DEFAULT 'any',
    rank                INTEGER      NOT NULL,
    from_market_id      BIGINT       NOT NULL,
    to_market_id        BIGINT       NOT NULL,
    commodity           VARCHAR(255) NOT NULL,
    buy_price           INTEGER      NOT NULL,
    sell_price          INTEGER      NOT NULL,
    profit              INTEGER      NOT NULL,
    supply              INTEGER      NOT NULL,
    demand              INTEGER      NOT NULL,
    distance_ly         REAL         NOT NULL,
    from_station_name   VARCHAR(255) NOT NULL,
    to_station_name     VARCHAR(255) NOT NULL,
    from_system_name    VARCHAR(255) NOT NULL,
    to_system_name      VARCHAR(255) NOT NULL,
    from_max_pad        CHAR(1),
    to_max_pad          CHAR(1),
    from_allegiance     VARCHAR(50),
    to_allegiance       VARCHAR(50),
    cached_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pad_filter, rank)
);

CREATE TABLE cached_trade_loops (
    pad_filter          VARCHAR(3)   NOT NULL DEFAULT 'any',
    rank                INTEGER      NOT NULL,
    market_id_a         BIGINT       NOT NULL,
    market_id_b         BIGINT       NOT NULL,
    commodity_out       VARCHAR(255) NOT NULL,
    buy_price_out       INTEGER      NOT NULL,
    sell_price_out      INTEGER      NOT NULL,
    profit_out          INTEGER      NOT NULL,
    commodity_back      VARCHAR(255) NOT NULL,
    buy_price_back      INTEGER      NOT NULL,
    sell_price_back     INTEGER      NOT NULL,
    profit_back         INTEGER      NOT NULL,
    total_profit        INTEGER      NOT NULL,
    distance_ly         REAL         NOT NULL,
    station_name_a      VARCHAR(255) NOT NULL,
    station_name_b      VARCHAR(255) NOT NULL,
    system_name_a       VARCHAR(255) NOT NULL,
    system_name_b       VARCHAR(255) NOT NULL,
    max_pad             CHAR(1),
    allegiance_a        VARCHAR(50),
    allegiance_b        VARCHAR(50),
    supply_out          INTEGER      NOT NULL DEFAULT 0,
    supply_back         INTEGER      NOT NULL DEFAULT 0,
    demand_out          INTEGER      NOT NULL DEFAULT 0,
    demand_back         INTEGER      NOT NULL DEFAULT 0,
    cached_at           TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    PRIMARY KEY (pad_filter, rank)
);

-- ── Migration: add event_timestamp to existing databases ─────
-- Run these ALTER TABLE statements on an existing database
-- (skip if applying schema.sql fresh to a new database).
--
-- ALTER TABLE journal_events    ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ;
-- ALTER TABLE star_systems  ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE stations      ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE star           ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE body           ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE commodity_listening ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE modul_listening     ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
-- ALTER TABLE ship_listening      ADD COLUMN IF NOT EXISTS event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z';
--
-- ── Migration: add allegiance columns to trade cache ─────────
-- ALTER TABLE cached_trade_routes ADD COLUMN IF NOT EXISTS from_allegiance VARCHAR(50);
-- ALTER TABLE cached_trade_routes ADD COLUMN IF NOT EXISTS to_allegiance   VARCHAR(50);
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS allegiance_a    VARCHAR(50);
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS allegiance_b    VARCHAR(50);
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS supply_out      INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS supply_back     INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS demand_out      INTEGER NOT NULL DEFAULT 0;
-- ALTER TABLE cached_trade_loops  ADD COLUMN IF NOT EXISTS demand_back     INTEGER NOT NULL DEFAULT 0;
--
-- ── Migration: clean up stale faction states and conflicts ───────────────────
-- faction_states accumulated rows without ever clearing them on re-visit,
-- so ended wars/elections left ghost states. Keep only the states that match
-- the faction's current journal_id (i.e. the most-recent visit snapshot).
--
-- DELETE FROM faction_states fs
-- WHERE fs.journal_id != (
--     SELECT f.journal_id FROM factions f
--     WHERE f.name = fs.faction_name AND f.system_address = fs.system_address
-- );
--
-- conflicts used ON CONFLICT DO NOTHING on a BIGSERIAL pk so it just stacked
-- rows forever. fetch_conflict used LIMIT 1 (= oldest row). Keep only the
-- most-recently inserted row per system, remove the rest.
--
-- DELETE FROM conflicts
-- WHERE id NOT IN (
--     SELECT MAX(id) FROM conflicts GROUP BY system_address
-- );
--
-- Wars in Elite Dangerous last days, not months. Delete any conflict data
-- older than 30 days as it is guaranteed to be stale.
--
-- DELETE FROM conflicts c
-- USING journal_events je
-- WHERE je.id = c.journal_id
--   AND je.event_timestamp < NOW() - INTERVAL '30 days';
--
-- ── Migration: add carrier_name column to stations ───────────────────────────
-- ALTER TABLE stations ADD COLUMN IF NOT EXISTS carrier_name VARCHAR(255);
--
-- ── Migration: add missing system_address indexes (bodies query) ─────────────
-- CREATE INDEX IF NOT EXISTS idx_ring_system           ON ring (system_address);
-- CREATE INDEX IF NOT EXISTS idx_parents_system        ON parents (system_address);
-- CREATE INDEX IF NOT EXISTS idx_planet_material_system ON planet_material (system_address);
--
-- ── Migration: widen cost/basevalue to BIGINT (ED prices exceed INTEGER range) ─
-- ALTER TABLE modul_listening ALTER COLUMN cost      TYPE BIGINT;
-- ALTER TABLE ship_listening  ALTER COLUMN basevalue TYPE BIGINT;;

-- ── Server tick tracking ─────────────────────────────────────
-- Each row is one detected BGS server tick.
-- tick_hour = unix_epoch_seconds / 3600, used for dedup without an expression index.
CREATE TABLE IF NOT EXISTS server_ticks (
    id           BIGSERIAL PRIMARY KEY,
    tick_time    TIMESTAMPTZ NOT NULL,
    system_count INTEGER     NOT NULL,
    detected_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tick_hour    BIGINT      NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_server_ticks_tick_hour ON server_ticks (tick_hour);

-- Migration (run on existing databases):
-- ALTER TABLE server_ticks ADD COLUMN IF NOT EXISTS tick_hour BIGINT NOT NULL DEFAULT 0;
-- UPDATE server_ticks SET tick_hour = extract(epoch from tick_time)::bigint / 3600 WHERE tick_hour = 0;
-- CREATE UNIQUE INDEX IF NOT EXISTS idx_server_ticks_tick_hour ON server_ticks (tick_hour);
