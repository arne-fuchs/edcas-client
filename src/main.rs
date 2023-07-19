extern crate core;

use eframe::{egui, HardwareAcceleration};
use std::sync::Mutex;

use crate::app::explorer::icons::{BodyIcons, PlanetSignalIcons, Symbols};
use crate::app::EliteRustClient;
use crate::egui::Vec2;

mod app;

#[macro_use]
extern crate lazy_static;
lazy_static! {
    static ref ICON_BODY_SIGNAL: Mutex<PlanetSignalIcons> = Mutex::new(PlanetSignalIcons::default());
    static ref ICON_BODY: Mutex<BodyIcons> = Mutex::new(BodyIcons::default());
    static ref ICON_SYMBOL: Mutex<Symbols> = Mutex::new(Symbols::default());
}

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = true;
    native_options.hardware_acceleration = HardwareAcceleration::Preferred;
    native_options.initial_window_size = Option::from(Vec2::new(1400.0, 800.0));
    eframe::run_native("ED: Commander Assistant System", native_options, Box::new(|_cc| Box::new(EliteRustClient::default()))).expect("Program panicked");
}
