CREATE TABLE system_population_history (
    id BIGSERIAL PRIMARY KEY,
    system_address BIGINT NOT NULL,
    population BIGINT NOT NULL,
    event_timestamp TIMESTAMPTZ NOT NULL
);
CREATE INDEX idx_sph_lookup ON system_population_history (system_address, event_timestamp DESC);
