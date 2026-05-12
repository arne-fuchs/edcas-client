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

## Installation

### Debian Based

Simply download the .deb file from the <a href="https://github.com/arne-fuchs/edcas-client/releases">release</a> tab and installit via dpkg:

```bash
sudo dpkg -i edcas-client.deb
```

### Arch

#### Aur:

```
paru -S edcas-client-bin
```
#### makepkg:

Download the PKGBUILD file.

In the same folder run:
```
makepkg -i
```

# Build it yourself

## Build Requirements

* Around 10 GB of disk space required
* <a href=https://www.rust-lang.org/tools/install >Rust with cargo</a>

### Debian based

```bash
sudo apt install cmake pkg-config build-essential git libwayland-dev libglib2.0-dev libgdk3.0-cil-dev libappindicator3-dev libsoup-3.0-dev libwebkit2gtk-4.1-dev libxdo-dev
```

## Standard directories

Edcas will first look into some standard directories before falling back into the current directory.
It might try to copy the files to the desired places.

| File                       | Look-up                                  | Fallback                |
|----------------------------|------------------------------------------|-------------------------|
| settings-example.json      | /etc/edcas-client/settigns-example.json  | ./settings-example.json |
| settings.json              | $HOME/.config/edcas-client/settings.json | ./settings.json         |
| Assets like materials.json | /usr/share/edcas-client/                 | ./                      |


## Using it with <a href=https://github.com/rfvgyhn/min-ed-launcher>min-ed-launcher</a>

Go and first build the project with
```bash
dx build --fullstack --release
```

Then edit your min-ed-launcher config:

```bash
nano ~/.config/min-ed-launcher/settings.json
```

and add this to your processes:

```json
"processes": [
        {
          "fileName": "/PATHTOEDCAS/edcas-client",
          "arguments": ""
        }
    ],
```

If you installed edcas-client over a package, you can find the binary in /usr/bin/edcas-client.

## Suggestions, Ideas & Bug Reports
Feel free to contact me for feature requests on Discord: frank_the_fish or use the issue feature.

For bugs, you can use the issue feature on GitHub.
