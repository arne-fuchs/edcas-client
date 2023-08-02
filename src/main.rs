extern crate core;

use std::env;
use std::str::FromStr;
use eframe::{egui, HardwareAcceleration};

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

    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = true;
    native_options.hardware_acceleration = HardwareAcceleration::Preferred;
    native_options.initial_window_size = Option::from(Vec2::new(width, height));
    eframe::run_native("ED: Commander Assistant System", native_options, Box::new(|_cc| client)).expect("Program panicked");
}
