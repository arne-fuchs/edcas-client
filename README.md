![logo](assets/graphics/logo/edcas.png)

# Elite Dangerous Commander Assistant System

Shows system information compact for explorer.

<h2>Gotta see all those planets!</h2>

![Screenshot of explorer panel](assets/graphics/screenshots/explorer-screenshot.png "Explorer Panel")

<h2>Track your materials!</h2>

![Screenshot of materials panel](assets/graphics/screenshots/materials-screenshot.png "Materials Panel")

<h2>And much more!</h2>

## Features

* System and its data represented in a compact view
* List signals found in system to find Raxxla ASAP
* Get help mining by showing relevant data
* Keep track of your materials
* Keep track of your constructions you'll never finish
* Keep track of what the feds are doing in the news tab
* Written in rust so you know its good
* All open source

---

## Running the client

### Install a pre-built package

**Debian / Ubuntu**

Download the `.deb` from the [releases](https://github.com/arne-fuchs/edcas-client/releases) page and install:

```bash
sudo dpkg -i edcas-client.deb
```

**Arch Linux (AUR)**

```bash
paru -S edcas-client-bin
```

Or with `makepkg` — download the `PKGBUILD` and run:

```bash
makepkg -i
```

### Build from source

**Requirements**

* Rust (stable) — [install](https://www.rust-lang.org/tools/install)
* ~10 GB disk space for the build cache

**System dependencies (Debian / Ubuntu)**

```bash
sudo apt install cmake pkg-config build-essential git \
    libwayland-dev libglib2.0-dev libgdk3.0-cil-dev \
    libappindicator3-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev libxdo-dev
```

**Build and run**

```bash
git clone https://github.com/arne-fuchs/edcas-client
cd edcas-client
cargo run --release
```

### Run in the browser (WebAssembly)

The client can also be compiled to WebAssembly and served as a static web page using [xterm.js](https://xtermjs.org/) as the terminal emulator. File I/O and journal reading are disabled; the search tabs (Stations, Carriers, Factions, Construction) work via async HTTP.

**Requirements**

* `wasm-pack` — `cargo install wasm-pack`
* Any static file server (e.g. Python's built-in one)

**Build**

```bash
./web/build.sh
# equivalent: wasm-pack build --target web --out-dir web/pkg
```

**Serve**

```bash
cd web
python3 -m http.server 8080
# open http://localhost:8080
```

Settings and pinned entries are stored in `localStorage` and survive page reloads.

### Configuration files

| File | Default location | Fallback |
|---|---|---|
| `settings.json` | `$HOME/.config/edcas-client/settings.json` | `./settings.json` |
| Assets (`materials.json`, …) | `/usr/share/edcas-client/` | `./` |

### Launch alongside the game (min-ed-launcher)

Add to your [min-ed-launcher](https://github.com/rfvgyhn/min-ed-launcher) config at `~/.config/min-ed-launcher/settings.json`:

```json
"processes": [
    {
        "fileName": "/usr/bin/edcas-client",
        "arguments": ""
    }
]
```

---

## Running the server

The server (`edcas-server`) listens to the [Elite Dangerous Data Network (EDDN)](https://github.com/EDCD/EDDN) ZeroMQ stream, ingests it into PostgreSQL, and exposes a REST API that the client queries for search results.

You only need to run the server yourself if you want to host your own instance. The public instance at `https://edcas.de` is used by default.

### Requirements

* PostgreSQL 14+
* `libzmq` (ZeroMQ 4.x)

**Debian / Ubuntu**

```bash
sudo apt install libzmq3-dev pkg-config
```

### Option A — Docker Compose (recommended)

This is the easiest way. From the repository root:

```bash
docker compose up --build
```

This starts PostgreSQL (with the schema applied automatically on first boot) and `edcas-server` together. The API is available at `http://localhost:3000`.

Data is persisted in `../edcas-data/postgres-data` relative to the repository root.

### Option B — Run manually

**1. Create the database**

```bash
createdb edcas
psql -d edcas -f crates/edcas-server/schema.sql
```

**2. Set environment variables**

| Variable | Default | Description |
|---|---|---|
| `DB_HOST` | `localhost` | PostgreSQL host |
| `DB_PORT` | `5432` | PostgreSQL port |
| `DB_USER` | *(required)* | PostgreSQL user |
| `DB_PASSWORD` | *(required)* | PostgreSQL password |
| `DB_NAME` | `edcas` | PostgreSQL database |
| `API_PORT` | `3000` | Port for the REST API |
| `EDDN_URL` | `tcp://eddn.edcd.io:9500` | EDDN ZeroMQ endpoint |
| `RUST_LOG` | — | Log filter, e.g. `edcas_server=info` |

**3. Build and run**

```bash
export DB_USER=edcas
export DB_PASSWORD=edcas

cargo run --release --bin edcas-server
```

### Pointing the client at your own server

In the client, go to the **Settings** tab → set the API URL to your server address, e.g.:

```
http://localhost:3000
```

### API endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/api/v1/systems/:address` | Star system metadata |
| `GET` | `/api/v1/systems/:address/bodies` | All bodies in a system |
| `GET` | `/api/v1/stations?name=&system_name=&market_id=&limit=` | Station search |
| `GET` | `/api/v1/carriers?name=&callsign=&system_name=&market_id=&limit=` | Fleet carrier search |
| `GET` | `/api/v1/factions?name=&limit=` | Faction search |
| `GET` | `/api/v1/construction-depots?name=&system_name=&market_id=&limit=` | Construction depot search |
| `POST` | `/api/v1/construction-depots` | Submit construction depot data |
| `POST` | `/api/v1/journal/event` | Upload a journal event |

All responses are JSON.

---

## Suggestions, Ideas & Bug Reports

Feel free to contact me for feature requests on Discord: `frank_the_fish` or use the issue tracker.

For bugs, please use the [GitHub issues](https://github.com/arne-fuchs/edcas-client/issues) page.
