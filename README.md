# EDCAS
### Elite Dangerous Commander Assistant System

Shows system information compact for explorer.

<h2>Gotta see all those planets!</h2>

![Screenshot of explorer panel](graphics/screenshots/explorer-screenshot.jpg "Explorer Panel")

<h2>Track your materials!</h2>

![Screenshot of materials panel](graphics/screenshots/materials-screenshot.jpg "Materials Panel")

<h2>Make it truly yours!</h2>

![Screenshot of settings panel](graphics/screenshots/settings-screenshot.jpg "Settings Panel")

<h2>And much more!</h2>

## Features

* System and its data represented in a compact view
* List signals found in system to find Raxxla ASAP
* Get help mining by showing relevant data
* Keep track of your materials
* Apply graphic override configurations (with presets or your own!)
* Always keep track of what the feds are doing in the news tab
* Written in rust so you know its good

## Build Requirements


### Debian based
```bash
sudo apt install cmake cargo pkg-config libasound2-dev libfontconfig1-dev libclang-dev libssl git
```
### Arch based
``` 
cargo alsa-lib fontconfig clang rocksdb
```

* gcc 12 required
* <a href=https://www.rust-lang.org/tools/install >Rust with cargo</a>


## Building

Clone the repo

```bash
git clone https://github.com/arne-fuchs/edcas-client.git
```

cd into it and build it

```bash
cd edcas-client && cargo build
```

## Using it with <a href=https://github.com/rfvgyhn/min-ed-launcher>min-ed-launcher</a>

Go and first build the project with
```bash
cargo build
```

Then edit your min-ed-launcher config:

```bash
nano ~/.config/min-ed-launcher/settings.json
```

and add this to your processes:

```json
"processes": [
        {
          "fileName": "/PATHTOPROJECT/edcas-client/start.sh",
          "arguments": ""
        }
    ],
```


## Suggestions, Ideas & Bug Reports
Feel free to contact me for feature requests on Discord: frank_the_fish or use the issue feature.

For bugs, you can use the issue feature on GitHub.
