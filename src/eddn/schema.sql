CREATE TABLE journal_events (
    id BIGSERIAL PRIMARY KEY,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    data JSONB NOT NULL,
    parsed BOOL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE faction_name (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE power (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE economy_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE allegiance (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE government (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE security (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE happiness (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE star_systems (
    system_address BIGINT PRIMARY KEY NOT NULL,
    event_id BIGINT REFERENCES journal_events(id),
    name VARCHAR(255) UNIQUE NOT NULL,
    x REAL,
    y REAL,
    z REAL,
    allegiance INTEGER REFERENCES allegiance(id),
    economy INTEGER REFERENCES economy_type(id),
    second_economy INTEGER REFERENCES economy_type(id),
    government INTEGER REFERENCES government(id),
    security INTEGER REFERENCES security(id),
    population BIGINT,
    controlling_power INTEGER REFERENCES power(id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE volcanism (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE atmosphere (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_class (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE atmosphere_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE terraform_state (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE body (
    id INTEGER NOT NULL,
    system_address BIGINT REFERENCES star_systems(system_address),
    name VARCHAR(255) UNIQUE NOT NULL,
    mass_em REAL,
    radius REAL,
    landable BOOL,
    axial_tilt REAL,
    periapsis REAL,
    tidal_lock BOOL,
    volcanism INTEGER REFERENCES volcanism(id),
    mapped BOOL,
    atmosphere INTEGER REFERENCES atmosphere(id),
    mean_anomaly REAL,
    planet_class INTEGER REFERENCES planet_class(id),
    eccentricity REAL,
    ascending_node REAL,
    orbital_period REAL,
    semi_major_axis REAL,
    atmosphere_type INTEGER REFERENCES atmosphere_type(id),
    rotation_period REAL,
    surface_gravity REAL,
    terraform_state INTEGER REFERENCES terraform_state(id),
    surface_pressure REAL,
    orbital_inclination REAL,
    surface_temperature REAL,
    distance REAL,
    journal_id BIGINT REFERENCES journal_events(id),
    PRIMARY KEY (id,system_address)
);

CREATE TABLE planet_composition_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_composition (
    composition_type INTEGER REFERENCES planet_composition_type(id),
    body_id INTEGER NOT NULL,
    system_address BIGINT NOT NULL,
    percent REAL NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    FOREIGN KEY (body_id,system_address) REFERENCES body(id,system_address)
);

CREATE TABLE atmosphere_composition (
    atmosphere_type INTEGER REFERENCES atmosphere_type(id),
    body_id INTEGER,
    system_address BIGINT,
    percent REAL NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    FOREIGN KEY (body_id,system_address) REFERENCES body(id,system_address)
);

CREATE TABLE material_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE planet_material (
    material_type INTEGER REFERENCES material_type(id),
    percent REAL NOT NULL,
    body_id INTEGER,
    system_address BIGINT,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    FOREIGN KEY (body_id,system_address) REFERENCES body(id,system_address)
);

CREATE TABLE star (
    id INTEGER NOT NULL,
    system_address BIGINT REFERENCES star_systems(system_address),
    name VARCHAR(255) UNIQUE NOT NULL,
    age_my INTEGER,
    radius REAL,
    star_type VARCHAR(3),
    subclass INTEGER,
    axial_tilt REAL,
    luminosity VARCHAR(3),
    stellar_mass REAL,
    rotation_period REAL,
    absolut_magnitude REAL,
    surface_temperature REAL,
    distance REAL,
    journal_id BIGINT REFERENCES journal_events(id),
    PRIMARY KEY (id,system_address)
);

CREATE TABLE parents (
    type VARCHAR(20),
    parent_id INTEGER,
    body_id INTEGER,
    system_address BIGINT,
    journal_id BIGINT REFERENCES journal_events(id),
    PRIMARY KEY (parent_id,body_id,system_address)
);

CREATE TABLE station_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE factions (
    name INTEGER REFERENCES faction_name(id),
    system_address BIGINT REFERENCES star_systems(system_address),
    government INTEGER REFERENCES government(id),
    allegiance INTEGER REFERENCES allegiance(id),
    happiness INTEGER REFERENCES happiness(id),
    influence REAL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (name,system_address)
);

CREATE TABLE stations(
    market_id BIGINT PRIMARY KEY NOT NULL,
    system_address BIGINT REFERENCES star_systems(system_address),
    body_id INTEGER,
    name VARCHAR(255) UNIQUE NOT NULL,
    type INTEGER REFERENCES station_type(id),
    faction_name INTEGER REFERENCES faction_name(id),
    government INTEGER REFERENCES government(id),
    economy INTEGER REFERENCES economy_type(id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    FOREIGN KEY (faction_name,system_address) REFERENCES factions(name,system_address),
    FOREIGN KEY (body_id,system_address) REFERENCES body(id,system_address)
);

CREATE TABLE commodity_name (
    id SERIAL PRIMARY KEY NOT NULL,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE commodity_listening (
    commodity_name INTEGER REFERENCES commodity_name(id),
    market_id BIGINT REFERENCES stations(market_id),
    buy_price INTEGER,
    demand INTEGER,
    demand_bracket INTEGER,
    mean_price INTEGER,
    sell_price INTEGER,
    stock INTEGER,
    stock_bracket INTEGER,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (commodity_name, market_id)
);

CREATE TABLE ship_name (
    id SERIAL PRIMARY KEY NOT NULL,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE ship_listening (
    ship_name INTEGER REFERENCES ship_name(id),
    market_id BIGINT REFERENCES stations(market_id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE modul_name (
    id SERIAL PRIMARY KEY NOT NULL,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE modul_listening (
    modul_name INTEGER REFERENCES modul_name(id),
    market_id BIGINT REFERENCES stations(market_id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE landing_pads_types(
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE station_services_types (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE station_landing_pads(
    market_id BIGINT REFERENCES stations(market_id),
    landing_pads_type INTEGER REFERENCES landing_pads_types(id),
    count INTEGER,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (market_id,landing_pads_type)
);
CREATE TABLE station_economies (
    id INTEGER REFERENCES economy_type(id),
    market_id BIGINT REFERENCES stations(market_id),
    proportion real NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (id,market_id)
);

CREATE TABLE station_services (
    id INTEGER REFERENCES station_services_types(id),
    market_id BIGINT REFERENCES stations(market_id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (id,market_id)
);

CREATE TABLE faction_state_name (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);
CREATE TYPE faction_state_state AS ENUM ('pending', 'active', 'recovering');
CREATE TABLE faction_states (
    state_name INTEGER REFERENCES faction_state_name(id),
    state_state faction_state_state NOT NULL,
    trend REAL,
    faction INTEGER REFERENCES faction_name(id),
    system_address BIGINT REFERENCES star_systems(system_address),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    PRIMARY KEY (faction,system_address,state_name)
);

CREATE TABLE war_type (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

CREATE TABLE conflict_faction_status (
    id SERIAL PRIMARY KEY NOT NULL,
    stake VARCHAR(255),
    won_days INTEGER,
    name INTEGER NOT NULL REFERENCES faction_name(id),
    system_address BIGINT REFERENCES star_systems(system_address),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL,
    FOREIGN KEY (name, system_address) REFERENCES factions(name, system_address)
);
CREATE TABLE conflict_status (
    id SERIAL PRIMARY KEY,
    value VARCHAR(255) UNIQUE NOT NULL,
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);
CREATE TABLE conflicts (
    system_address BIGINT REFERENCES star_systems(system_address),
    faction1 INTEGER REFERENCES faction_name(id),
    faction2 INTEGER REFERENCES faction_name(id),
    war_type INTEGER REFERENCES war_type(id),
    status INTEGER REFERENCES conflict_status(id),
    journal_id BIGINT REFERENCES journal_events(id) NOT NULL
);

--- Indexing
