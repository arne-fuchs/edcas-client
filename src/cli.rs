use std::any::Any;

use crate::edcas::{
    self,
    backend::evm::journal_uploader,
    settings::{self, Settings},
    EliteRustClient,
};
use ethers::prelude::*;

pub fn upload_journal(client: EliteRustClient) {
    let journal_path = client
        .settings
        .lock()
        .unwrap()
        .journal_reader_settings
        .journal_directory
        .clone();

    let evm_settings = &client.settings.lock().unwrap().evm_settings;

    // start evm uploader thread
    let (mut progress_bus_reader, total) = journal_uploader::initialize(evm_settings, journal_path);
    println!("Uploading {} logs", total);

    // loop until uploading is done
    loop {
        match progress_bus_reader.recv() {
            Ok(index) => {
                if index <= 0 {
                    println!("done uploading");
                    break;
                }
                print!("{} ", total - index);
            }
            Err(err) => {
                println!("\npanicked: {}", err);
                break;
            }
        }
    }
}

pub fn set_sc_address(smart_contract_address: String, client: EliteRustClient) {
    let _ = smart_contract_address
        .parse::<Address>()
        .unwrap_or_else(|_| panic!("Address is incorrect"));

    let mut settings = client.settings.lock().unwrap();

    settings.evm_settings.smart_contract_address = smart_contract_address;
    settings.save_settings_to_file();
}

pub fn set_journal_path(journal_path: String) {
    let mut settings = edcas::settings::Settings::default();
    settings.journal_reader_settings.journal_directory = journal_path;
    settings.save_settings_to_file();
}

pub fn set_settings_path(new_settings_path: String) {
    let mut settings = edcas::settings::Settings {
        settings_path: new_settings_path,
        ..Default::default() // did that becaouse clippy was angry i reassigned a field later
    };
    //settings.settings_path = new_settings_path;
    settings.save_settings_to_file();
}

pub fn set_graphics_path(new_graphics_path: String) {
    let mut settings = edcas::settings::Settings::default();
    settings.graphic_editor_settings.graphics_directory = new_graphics_path;
    settings.save_settings_to_file();
}
