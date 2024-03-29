extern crate core;

use eframe::egui::{IconData, Pos2, ViewportBuilder};
use eframe::{egui, HardwareAcceleration};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::app::EliteRustClient;

mod app;
mod tui;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut wpos: f32 = -1.0;
    let mut hpos: f32 = -1.0;
    let mut fullscreen = false;
    let mut maximized = false;

    for i in 0..args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--wposition" => {
                wpos = f32::from_str(args[i + 1].as_str())
                    .expect(format!("Wrong argument for width: {} ", &args[i + 1]).as_str());
            }
            "--hposition" => {
                hpos = f32::from_str(args[i + 1].as_str())
                    .expect(format!("Wrong argument for width: {} ", &args[i + 1]).as_str());
            }
            "--fullscreen" => {
                fullscreen = true;
            }
            "--maximized" => {
                maximized = true;
            }
            "--help" => {
                let ascii_art = r#"
  ______    ____      ____       _       ______
 |  ____|  |  _ \   / ____|     / \     /  ____|
 | |__     | | | |  | |        / _ \    |  (___
 |  __|    | | | |  | |       / /_\ \   \___   \
 | |____   | |_| |  | |____  /  / \  \   ____) |
 |______|  |____/   \_____| /__/   \__\ |_____/

"#;
                println!("{}", ascii_art);
                println!("Here is a list of all commands:\n");
                println!("--version\tPrints the current version of edcas");
                println!("--height <f32>\tSets the height for the edcas gui");
                println!("--width <f32>\tSets the width for the edcas gui");
                return;
            }
            "--tui" => {
                let client = EliteRustClient::default();
                tui::draw_tui(client).unwrap();
                return;
            }
            _ => {}
        }
    }

    let client = Box::<EliteRustClient>::default();

    //App icon
    let mut image = image::open("graphics\\logo\\edcas_128.png");
    if cfg!(target_os = "linux") {
        match image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas_128.png") {
            Ok(_) => {
                image = image::open("/usr/share/edcas-client/graphics/logo/edcas_128.png");
            }
            Err(_) => {
                image = image::open("graphics/logo/edcas.png");
            }
        }
    }

    let (icon_rgba, icon_width, icon_height) = {
        let image = image.unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    };

    let mut viewport = ViewportBuilder::default()
        .with_icon(Arc::new(icon))
        .with_app_id("edcas-client")
        .with_title("ED: Commander Assistant System")
        .with_decorations(true)
        .with_taskbar(true)
        .with_resizable(true)
        .with_maximize_button(true)
        .with_minimize_button(true)
        .with_close_button(true)
        .with_titlebar_shown(true);

    if wpos > 0.0 && hpos > 0.0 {
        viewport = viewport.with_position(Pos2::new(wpos, hpos));
    }
    if fullscreen {
        viewport = viewport.with_fullscreen(true);
    }
    if maximized {
        viewport = viewport.with_maximized(true);
    }

    let native_options = eframe::NativeOptions {
        hardware_acceleration: HardwareAcceleration::Preferred,
        persist_window: true,
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "ED: Commander Assistant System",
        native_options,
        Box::new(|_cc| client),
    )
    .expect("Program panicked");
}
