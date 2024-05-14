use crate::edcas::{backend::evm::journal_uploader, EliteRustClient};
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
