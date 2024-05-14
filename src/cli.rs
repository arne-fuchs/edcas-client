use ethers::prelude::*;

use crate::edcas::{backend::evm::journal_uploader, EliteRustClient};

pub fn upload_journal(mut client: EliteRustClient) {
    let journal_path = client
        .settings
        .lock()
        .unwrap()
        .journal_reader_settings
        .journal_directory
        .clone();

    let evm_settings = &client.settings.lock().unwrap().evm_settings;
    let (mut progress_bus_reader, total) = journal_uploader::initialize(evm_settings, journal_path);

    println!("Uploading {} logs", total);

    loop {
        match progress_bus_reader.recv() {
            Ok(index) => {
                if index <= 0 {
                    println!("done uploading");
                    break;
                }
                println!("{}", total - index);
            }
            Err(err) => {
                println!("panicked: {}", err);
                break;
            }
        }
    }

    return;
    //TODO: start evm uploader thread
    //TODO: start journal reader thread
    //TODO: feed data from journal reader to evm uploader

    //TODO: ask Frank what i have to do and how to actually do that.
}

pub fn set_sc_address(smart_contract_address: String, client: EliteRustClient) {
    let addr = smart_contract_address
        .parse::<Address>()
        .unwrap_or_else(|_| panic!("Address is incorrect"));
    println!("nothing here yet");
    //TODO: take that sc_address and put it either directly in json or in the client.settings and
    //then call save
    return;
}
