extern crate core;

use std::env;
use std::str::FromStr;
use eframe::{egui, HardwareAcceleration, IconData};

use crate::app::EliteRustClient;
use crate::egui::Vec2;

mod app;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut width :f32 = 1400.0;
    let mut height :f32 = 800.0;

    for i in 0..args.len() {
        match args[i].as_str() {
            "--version" => {
                println!("{}",env!("CARGO_PKG_VERSION"));
                return;
            }
            "--width" => {
                width = f32::from_str(args[i+1].as_str()).expect(format!("Wrong argument for width: {} ", &args[i+1]).as_str());
            }
            "--height" => {
                height = f32::from_str(args[i+1].as_str()).expect(format!("Wrong argument for width: {} ", &args[i+1]).as_str());
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
        let image = image.unwrap()
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    let icon = IconData{
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height
    };

    let mut native_options = eframe::NativeOptions::default();
    native_options.app_id = Some("edcas-client".to_string());
    native_options.icon_data = Some(icon);
    native_options.vsync = true;
    native_options.hardware_acceleration = HardwareAcceleration::Preferred;
    native_options.initial_window_size = Option::from(Vec2::new(width, height));
    eframe::run_native("ED: Commander Assistant System", native_options, Box::new(|_cc| client)).expect("Program panicked");
}
