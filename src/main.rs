#![allow(unreachable_code)]
extern crate core;

use eframe::egui::{IconData, Pos2, TextBuffer, ViewportBuilder};
use eframe::HardwareAcceleration;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use crate::edcas::EliteRustClient;

mod cli;
mod edcas;
#[cfg(feature = "eddn")]
mod eddn;
mod gui;
#[cfg(feature = "tui")]
mod tui;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut wpos: f32 = -1.0;
    let mut hpos: f32 = -1.0;
    let mut fullscreen = false;
    let mut maximized = false;
    let ascii_art = r#"
  ______    ____      ____       _       ______
 |  ____|  |  _ \   / ____|     / \     /  ____|
 | |__     | | | |  | |        / _ \    |  (___
 |  __|    | | | |  | |       / /_\ \   \___   \
 | |____   | |_| |  | |____  /  / \  \   ____) |
 |______|  |____/   \_____| /__/   \__\ |_____/

"#;

    for i in 0..args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--wposition" => {
                wpos = f32::from_str(args[i + 1].as_str())
                    .unwrap_or_else(|_| panic!("Wrong argument for width: {} ", &args[i + 1]));
            }
            "--hposition" => {
                hpos = f32::from_str(args[i + 1].as_str())
                    .unwrap_or_else(|_| panic!("Wrong argument for width: {} ", &args[i + 1]));
            }
            "--fullscreen" => {
                fullscreen = true;
            }
            "--maximized" => {
                maximized = true;
            }
            "--help" => {
                println!("{}", ascii_art);
                println!("--version\t\tPrints the current version of edcas");
                println!("--height <f32>\t\tSets the height for the edcas gui");
                println!("--width <f32>\t\tSets the width for the edcas gui");
                println!("--set-sc-address\tSet the smart contract address");
                println!("--upload-journal\tUpload Journal to EDCAS network");
                #[cfg(feature = "tui")]
                println!("--tui\t\t\tStart edcas in tui mode");
                #[cfg(feature = "eddn")]
                println!("--eddn\t\tStart EDCAS with EDDN support");
                return;
            }
            "--set-sc-address" => {
                let client = EliteRustClient::default();
                let new_smart_contract_address = String::from_str(args[i + 1].as_str())
                    .unwrap_or_else(|_| panic!("Wrong argument for SC Address: {}", &args[i + 1]));
                cli::set_sc_address(new_smart_contract_address, client);
                return;
            }
            "--upload-journal" => {
                let client = EliteRustClient::default();
                cli::upload_journal(client);
                return;
            }
            "--set-journal-path" => {
                let new_journal_path =
                    String::from_str(args[i + 1].as_str()).unwrap_or_else(|_| {
                        panic!("Wrong argument for Journal path: {}", &args[i + 1])
                    });
                cli::set_journal_path(new_journal_path);
                return;
            }
            "--set-graphics-path" => {
                let new_graphics_path =
                    String::from_str(args[i + 1].as_str()).unwrap_or_else(|_| {
                        panic!("Wrong argument for Graphics path: {}", &args[i + 1])
                    });
                cli::set_graphics_path(new_graphics_path);
                return;
            }
            "--set-settings-path" => {
                let new_settings_path =
                    String::from_str(args[i + 1].as_str()).unwrap_or_else(|_| {
                        panic!("Wrong argument for Settings path: {}", &args[i + 1])
                    });
                cli::set_settings_path(new_settings_path);
                return;
            }
            #[cfg(feature = "tui")]
            "--tui" => {
                let client = EliteRustClient::default();
                tui::draw_tui(client).unwrap();
                return;
            }
            #[cfg(feature = "eddn")]
            "--eddn" => {
                println!("{}", ascii_art);
                eddn::initialize();
                return;
            }
            _ => {}
        }
    }

    let client = Box::<EliteRustClient>::default();

    let source = include_bytes!("../graphics/logo/edcas_128.png");
    let image = image::load_from_memory(source);

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
