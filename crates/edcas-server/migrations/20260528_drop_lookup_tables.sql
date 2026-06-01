-- Drop 20 lookup/enum tables and store strings inline.
-- Column names stay the same except:
--   stations.type        → stations.station_type  (reserved word clash)
--   station_services.id  → station_services.service_type
-- Run with: psql -f 20260528_drop_lookup_tables.sql

BEGIN;

-- ── star_systems ─────────────────────────────────────────────
ALTER TABLE star_systems
  ADD COLUMN allegiance_new      VARCHAR,
  ADD COLUMN economy_new         VARCHAR,
  ADD COLUMN second_economy_new  VARCHAR,
  ADD COLUMN government_new      VARCHAR,
  ADD COLUMN security_new        VARCHAR,
  ADD COLUMN controlling_power_new VARCHAR;

UPDATE star_systems ss SET
  allegiance_new       = (SELECT value FROM allegiance      a  WHERE a.id  = ss.allegiance),
  economy_new          = (SELECT value FROM economy_type    e  WHERE e.id  = ss.economy),
  second_economy_new   = (SELECT value FROM economy_type    e  WHERE e.id  = ss.second_economy),
  government_new       = (SELECT value FROM government      g  WHERE g.id  = ss.government),
  security_new         = (SELECT value FROM security        s  WHERE s.id  = ss.security),
  controlling_power_new= (SELECT value FROM power           p  WHERE p.id  = ss.controlling_power);

ALTER TABLE star_systems
  DROP COLUMN allegiance       CASCADE,
  DROP COLUMN economy          CASCADE,
  DROP COLUMN second_economy   CASCADE,
  DROP COLUMN government       CASCADE,
  DROP COLUMN security         CASCADE,
  DROP COLUMN controlling_power CASCADE;

ALTER TABLE star_systems RENAME COLUMN allegiance_new       TO allegiance;
ALTER TABLE star_systems RENAME COLUMN economy_new          TO economy;
ALTER TABLE star_systems RENAME COLUMN second_economy_new   TO second_economy;
ALTER TABLE star_systems RENAME COLUMN government_new       TO government;
ALTER TABLE star_systems RENAME COLUMN security_new         TO security;
ALTER TABLE star_systems RENAME COLUMN controlling_power_new TO controlling_power;

-- ── stations ─────────────────────────────────────────────────
ALTER TABLE stations
  ADD COLUMN station_type_new VARCHAR,
  ADD COLUMN government_new   VARCHAR,
  ADD COLUMN economy_new      VARCHAR;

UPDATE stations s SET
  station_type_new = (SELECT value FROM station_type  st WHERE st.id = s.type),
  government_new   = (SELECT value FROM government     g  WHERE g.id  = s.government),
  economy_new      = (SELECT value FROM economy_type   e  WHERE e.id  = s.economy);

ALTER TABLE stations
  DROP COLUMN type       CASCADE,
  DROP COLUMN government CASCADE,
  DROP COLUMN economy    CASCADE;

ALTER TABLE stations RENAME COLUMN station_type_new TO station_type;
ALTER TABLE stations RENAME COLUMN government_new   TO government;
ALTER TABLE stations RENAME COLUMN economy_new      TO economy;

-- ── station_services ─────────────────────────────────────────
-- id was the FK column; rename to service_type and store the string value.
-- The PK was (id, market_id); we recreate it as (service_type, market_id).
ALTER TABLE station_services ADD COLUMN service_type_new VARCHAR;

UPDATE station_services ss SET
  service_type_new = (SELECT value FROM station_services_types sst WHERE sst.id = ss.id);

ALTER TABLE station_services DROP CONSTRAINT station_services_pkey;
ALTER TABLE station_services DROP COLUMN id CASCADE;
ALTER TABLE station_services RENAME COLUMN service_type_new TO service_type;
ALTER TABLE station_services ADD PRIMARY KEY (service_type, market_id);

-- ── station_economies ────────────────────────────────────────
-- PK is (market_id, economy_type); we rebuild it with VARCHAR.
ALTER TABLE station_economies ADD COLUMN economy_type_new VARCHAR;

UPDATE station_economies se SET
  economy_type_new = (SELECT value FROM economy_type e WHERE e.id = se.economy_type);

ALTER TABLE station_economies DROP CONSTRAINT station_economies_pkey;
ALTER TABLE station_economies DROP COLUMN economy_type CASCADE;
ALTER TABLE station_economies RENAME COLUMN economy_type_new TO economy_type;
ALTER TABLE station_economies ADD PRIMARY KEY (market_id, economy_type);

-- ── factions ─────────────────────────────────────────────────
ALTER TABLE factions
  ADD COLUMN government_new VARCHAR,
  ADD COLUMN allegiance_new VARCHAR,
  ADD COLUMN happiness_new  VARCHAR;

UPDATE factions f SET
  government_new = (SELECT value FROM government g WHERE g.id = f.government),
  allegiance_new = (SELECT value FROM allegiance a WHERE a.id = f.allegiance),
  happiness_new  = (SELECT value FROM happiness  h WHERE h.id = f.happiness);

ALTER TABLE factions
  DROP COLUMN government CASCADE,
  DROP COLUMN allegiance CASCADE,
  DROP COLUMN happiness  CASCADE;

ALTER TABLE factions RENAME COLUMN government_new TO government;
ALTER TABLE factions RENAME COLUMN allegiance_new TO allegiance;
ALTER TABLE factions RENAME COLUMN happiness_new  TO happiness;

-- ── faction_states ───────────────────────────────────────────
-- PK includes state (INT); rebuild as VARCHAR.
ALTER TABLE faction_states ADD COLUMN state_new VARCHAR;

UPDATE faction_states fs SET
  state_new = (SELECT value FROM faction_state_name fsn WHERE fsn.id = fs.state);

ALTER TABLE faction_states DROP CONSTRAINT faction_states_pkey;
ALTER TABLE faction_states DROP COLUMN state CASCADE;
ALTER TABLE faction_states RENAME COLUMN state_new TO state;
ALTER TABLE faction_states ADD PRIMARY KEY (faction_name, system_address, state);

-- ── conflicts ────────────────────────────────────────────────
ALTER TABLE conflicts ADD COLUMN war_type_new VARCHAR;

UPDATE conflicts c SET
  war_type_new = (SELECT value FROM war_type wt WHERE wt.id = c.war_type);

ALTER TABLE conflicts DROP COLUMN war_type CASCADE;
ALTER TABLE conflicts RENAME COLUMN war_type_new TO war_type;

-- ── body ─────────────────────────────────────────────────────
ALTER TABLE body
  ADD COLUMN planet_class_new     VARCHAR,
  ADD COLUMN volcanism_new        VARCHAR,
  ADD COLUMN atmosphere_new       VARCHAR,
  ADD COLUMN atmosphere_type_new  VARCHAR,
  ADD COLUMN terraform_state_new  VARCHAR;

UPDATE body b SET
  planet_class_new    = (SELECT value FROM planet_class    pc WHERE pc.id = b.planet_class),
  volcanism_new       = (SELECT value FROM volcanism        v  WHERE v.id  = b.volcanism),
  atmosphere_new      = (SELECT value FROM atmosphere       a  WHERE a.id  = b.atmosphere),
  atmosphere_type_new = (SELECT value FROM atmosphere_type at  WHERE at.id = b.atmosphere_type),
  terraform_state_new = (SELECT value FROM terraform_state  ts WHERE ts.id = b.terraform_state);

ALTER TABLE body
  DROP COLUMN planet_class    CASCADE,
  DROP COLUMN volcanism       CASCADE,
  DROP COLUMN atmosphere      CASCADE,
  DROP COLUMN atmosphere_type CASCADE,
  DROP COLUMN terraform_state CASCADE;

ALTER TABLE body RENAME COLUMN planet_class_new    TO planet_class;
ALTER TABLE body RENAME COLUMN volcanism_new       TO volcanism;
ALTER TABLE body RENAME COLUMN atmosphere_new      TO atmosphere;
ALTER TABLE body RENAME COLUMN atmosphere_type_new TO atmosphere_type;
ALTER TABLE body RENAME COLUMN terraform_state_new TO terraform_state;

-- ── atmosphere_composition ───────────────────────────────────
ALTER TABLE atmosphere_composition ADD COLUMN atmosphere_type_new VARCHAR;

UPDATE atmosphere_composition ac SET
  atmosphere_type_new = (SELECT value FROM atmosphere_type at WHERE at.id = ac.atmosphere_type);

ALTER TABLE atmosphere_composition DROP COLUMN atmosphere_type CASCADE;
ALTER TABLE atmosphere_composition RENAME COLUMN atmosphere_type_new TO atmosphere_type;

-- ── planet_material ──────────────────────────────────────────
ALTER TABLE planet_material ADD COLUMN material_type_new VARCHAR;

UPDATE planet_material pm SET
  material_type_new = (SELECT value FROM material_type mt WHERE mt.id = pm.material_type);

ALTER TABLE planet_material DROP COLUMN material_type CASCADE;
ALTER TABLE planet_material RENAME COLUMN material_type_new TO material_type;

-- ── planet_composition ───────────────────────────────────────
ALTER TABLE planet_composition ADD COLUMN composition_type_new VARCHAR;

UPDATE planet_composition pc SET
  composition_type_new = (SELECT value FROM planet_composition_type pct WHERE pct.id = pc.composition_type);

ALTER TABLE planet_composition DROP COLUMN composition_type CASCADE;
ALTER TABLE planet_composition RENAME COLUMN composition_type_new TO composition_type;

-- ── ring ─────────────────────────────────────────────────────
ALTER TABLE ring ADD COLUMN ring_class_new VARCHAR;

UPDATE ring r SET
  ring_class_new = (SELECT value FROM ring_class rc WHERE rc.id = r.ring_class);

ALTER TABLE ring DROP COLUMN ring_class CASCADE;
ALTER TABLE ring RENAME COLUMN ring_class_new TO ring_class;

-- ── star ─────────────────────────────────────────────────────
ALTER TABLE star ADD COLUMN star_type_new VARCHAR;

UPDATE star s SET
  star_type_new = (SELECT value FROM star_type st WHERE st.id = s.star_type);

ALTER TABLE star DROP COLUMN star_type CASCADE;
ALTER TABLE star RENAME COLUMN star_type_new TO star_type;

-- ── fss_body_signals ─────────────────────────────────────────
-- PK includes signal_type (INT); rebuild as VARCHAR.
ALTER TABLE fss_body_signals ADD COLUMN signal_type_new VARCHAR;

UPDATE fss_body_signals fbs SET
  signal_type_new = (SELECT value FROM signal_type st WHERE st.id = fbs.signal_type);

ALTER TABLE fss_body_signals DROP CONSTRAINT fss_body_signals_pkey;
ALTER TABLE fss_body_signals DROP COLUMN signal_type CASCADE;
ALTER TABLE fss_body_signals RENAME COLUMN signal_type_new TO signal_type;
ALTER TABLE fss_body_signals ADD PRIMARY KEY (body_id, system_address, signal_type);

-- ── saa_signals ──────────────────────────────────────────────
ALTER TABLE saa_signals ADD COLUMN signal_type_new VARCHAR;

UPDATE saa_signals ss SET
  signal_type_new = (SELECT value FROM signal_type st WHERE st.id = ss.signal_type);

ALTER TABLE saa_signals DROP CONSTRAINT saa_signals_pkey;
ALTER TABLE saa_signals DROP COLUMN signal_type CASCADE;
ALTER TABLE saa_signals RENAME COLUMN signal_type_new TO signal_type;
ALTER TABLE saa_signals ADD PRIMARY KEY (body_id, system_address, signal_type);

-- ── Drop all 20 lookup tables ────────────────────────────────
DROP TABLE allegiance;
DROP TABLE atmosphere;
DROP TABLE atmosphere_type;
DROP TABLE economy_type;
DROP TABLE faction_state_name;
DROP TABLE government;
DROP TABLE happiness;
DROP TABLE material_type;
DROP TABLE planet_class;
DROP TABLE planet_composition_type;
DROP TABLE power;
DROP TABLE ring_class;
DROP TABLE security;
DROP TABLE signal_type;
DROP TABLE star_type;
DROP TABLE station_services_types;
DROP TABLE station_type;
DROP TABLE terraform_state;
DROP TABLE volcanism;
DROP TABLE war_type;

COMMIT;
