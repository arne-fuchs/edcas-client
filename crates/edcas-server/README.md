# edcas-server

Backend service for [EDCAS](https://edcas.de) — Elite Dangerous Commander Assistant System.

It does two things concurrently:

1. **EDDN listener** — subscribes to the [Elite Dangerous Data Network](https://github.com/EDCD/EDDN) ZeroMQ stream, decompresses incoming messages, and persists them to PostgreSQL (star systems, bodies, stations, signals, market data).
2. **REST API** — serves that data to `edcas-client` and other consumers over HTTP.

## Requirements

- Rust 1.75+
- PostgreSQL 14+
- `libzmq` (ZeroMQ 4.x)

On Debian/Ubuntu:

```bash
sudo apt install libzmq3-dev pkg-config
```

## Database setup

Create a database and apply the schema:

```bash
createdb edcas
psql -d edcas -f schema.sql
```

## Configuration

All configuration is via environment variables:

| Variable | Default | Description |
|---|---|---|
| `DB_HOST` | `localhost` | PostgreSQL host |
| `DB_PORT` | `5432` | PostgreSQL port |
| `DB_USER` | *(required)* | PostgreSQL user |
| `DB_PASSWORD` | *(required)* | PostgreSQL password |
| `DB_NAME` | `edcas` | PostgreSQL database name |
| `API_PORT` | `3000` | Port for the REST API |
| `EDDN_URL` | `tcp://eddn.edcd.io:9500` | EDDN ZeroMQ endpoint |
| `RUST_LOG` | — | Log filter, e.g. `edcas_server=info` |

## Running locally

```bash
export DB_USER=edcas
export DB_PASSWORD=edcas

cargo run --release --bin edcas-server
```

## Running with Docker Compose

From the workspace root:

```bash
docker compose up --build
```

This starts PostgreSQL (with the schema applied automatically) and `edcas-server` together. The API is available at `http://localhost:3000`.

## API endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/systems/:address` | Star system metadata |
| `GET` | `/api/v1/systems/:address/bodies` | All bodies (planets + stars) in a system |
| `GET` | `/api/v1/stations?name=&system_name=&market_id=&limit=` | Station search |
| `GET` | `/api/v1/carriers?name=&callsign=&system_name=&limit=` | Fleet carrier search |

All responses are JSON. `:address` is the 64-bit `SystemAddress` from the Elite Dangerous journal.
