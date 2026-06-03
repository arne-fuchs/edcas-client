CREATE TABLE faction_influence_history (
    id BIGSERIAL PRIMARY KEY,
    faction_name VARCHAR NOT NULL,
    system_address BIGINT NOT NULL,
    influence REAL NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_fih_lookup ON faction_influence_history (faction_name, system_address, event_timestamp DESC);

CREATE TABLE commodity_price_history (
    id BIGSERIAL PRIMARY KEY,
    market_id BIGINT NOT NULL,
    name VARCHAR NOT NULL,
    buy_price INT NOT NULL,
    sell_price INT NOT NULL,
    stock INT NOT NULL,
    demand INT NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_cph_lookup ON commodity_price_history (market_id, name, event_timestamp DESC);
