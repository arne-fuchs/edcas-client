extern crate core;

use eframe::{egui, HardwareAcceleration};

use crate::app::EliteRustClient;
use crate::egui::Vec2;

mod app;

fn main() {
    let mut native_options = eframe::NativeOptions::default();
    native_options.vsync = true;
    native_options.hardware_acceleration = HardwareAcceleration::Preferred;
    native_options.initial_window_size = Option::from(Vec2::new(1400.0, 800.0));
    eframe::run_native("ED: Commander Assistant System", native_options, Box::new(|_cc| Box::<EliteRustClient>::default())).expect("Program panicked");
}
