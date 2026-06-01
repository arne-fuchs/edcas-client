-- ============================================================
-- EDCAS-EDDN canonical schema
-- Matches what crates/edcas-eddn writes exactly.
-- Apply to a fresh database with: psql -f schema.sql
-- ============================================================

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
    allegiance      VARCHAR(255),
    economy         VARCHAR(255),
    second_economy  VARCHAR(255),
    government      VARCHAR(255),
    security        VARCHAR(255),
    population      BIGINT,
    controlling_power VARCHAR(255),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

-- ── Factions ─────────────────────────────────────────────────

CREATE TABLE factions (
    name            VARCHAR(255) NOT NULL,
    system_address  BIGINT,
    government      VARCHAR(255),
    allegiance      VARCHAR(255),
    happiness       VARCHAR(255),
    influence       REAL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (name, system_address)
);

CREATE TABLE faction_states (
    faction_name    VARCHAR(255) NOT NULL,
    system_address  BIGINT NOT NULL,
    state           VARCHAR(255) NOT NULL,
    status          VARCHAR(50) NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (faction_name, system_address, state),
    FOREIGN KEY (faction_name, system_address) REFERENCES factions(name, system_address)
);

CREATE TABLE conflicts (
    id                  BIGSERIAL PRIMARY KEY,
    system_address      BIGINT,
    war_type            VARCHAR(255),
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
    star_type           VARCHAR(255),
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
    volcanism           VARCHAR(255),
    mapped              BOOL,
    atmosphere          VARCHAR(255),
    mean_anomaly        REAL,
    planet_class        VARCHAR(255),
    eccentricity        REAL,
    ascending_node      REAL,
    orbital_period      REAL,
    semi_major_axis     REAL,
    atmosphere_type     VARCHAR(255),
    rotation_period     REAL,
    surface_gravity     REAL,
    terraform_state     VARCHAR(255),
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
    ring_class      VARCHAR(255),
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
    atmosphere_type VARCHAR(255),
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    percent         REAL NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_material (
    material_type   VARCHAR(255),
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    percent         REAL NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_composition (
    composition_type VARCHAR(255),
    body_id          INTEGER NOT NULL,
    system_address   BIGINT NOT NULL,
    percent          REAL NOT NULL,
    journal_id       BIGINT REFERENCES journal_events(id) NOT NULL
);

-- ── FSS / SAA signal tables ──────────────────────────────────

CREATE TABLE fss_body_signals (
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    signal_type     VARCHAR(255) NOT NULL,
    count           INTEGER NOT NULL,
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (body_id, system_address, signal_type)
);

CREATE TABLE saa_signals (
    body_id         INTEGER NOT NULL,
    system_address  BIGINT NOT NULL,
    signal_type     VARCHAR(255) NOT NULL,
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
    station_type    VARCHAR(255),
    faction_name    VARCHAR(255),
    government      VARCHAR(255),
    economy         VARCHAR(255),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

CREATE TABLE station_services (
    service_type    VARCHAR(255) NOT NULL,
    market_id       BIGINT REFERENCES stations(market_id),
    journal_id      BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (service_type, market_id)
);

CREATE TABLE station_economies (
    market_id    BIGINT REFERENCES stations(market_id),
    economy_type VARCHAR(255) NOT NULL,
    proportion   REAL NOT NULL,
    journal_id   BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (market_id, economy_type)
);

CREATE TABLE station_landing_pads (
    market_id   BIGINT PRIMARY KEY REFERENCES stations(market_id),
    small       INTEGER,
    medium      INTEGER,
    large       INTEGER,
    journal_id  BIGINT REFERENCES journal_events(id) NOT NULL
);

-- ── Market data ──────────────────────────────────────────────

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

-- ── Pre-computed trade cache ─────────────────────────────────

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

-- ── Server tick tracking ─────────────────────────────────────

CREATE TABLE IF NOT EXISTS server_ticks (
    id           BIGSERIAL PRIMARY KEY,
    tick_time    TIMESTAMPTZ NOT NULL,
    system_count INTEGER     NOT NULL,
    detected_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tick_hour    BIGINT      NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_server_ticks_tick_hour ON server_ticks (tick_hour);
