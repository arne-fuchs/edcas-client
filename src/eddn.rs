use crate::edcas::settings::EvmSettings;
use bus::Bus;
use json::JsonValue;
use log::LevelFilter;
use std::{env, thread};

use crate::eddn::adapter::EddnAdapter;

mod adapter;

pub fn initialize() {
    println!("Initializing eddn adapter...");
    env_logger::Builder::new()
        .filter_level(LevelFilter::Info)
        .init();

    let mut bus_writer: Bus<JsonValue> = Bus::new(1000);
    let bus_reader = bus_writer.add_rx();

    let eddn = EddnAdapter { bus_writer };
    let evm_settings = EvmSettings {
        url: env::var("EVM_URL").unwrap(),
        n_timeout: env::var("DURATION_TIMEOUT")
            .unwrap_or("5".into())
            .parse()
            .unwrap_or(5),
        n_attempts: env::var("RETRY_TIMEOUT")
            .unwrap_or("100".into())
            .parse()
            .unwrap_or(100),
        allow_share_data: true,
        private_key: env::var("PRIVATE_KEY").unwrap(),
        smart_contract_address: env::var("SC_ADDRESS").unwrap(),
        contract: None,
        show_upload_data_window: false,
        journal_read_status: None,
    };
    let mut evm_interpreter =
        crate::edcas::backend::evm::journal_interpreter::initialize(bus_reader, &evm_settings);
    thread::Builder::new()
        .name("edcas-evm-interpreter".into())
        .spawn(move || evm_interpreter.run_loop())
        .expect("Can't spawn eddn thread");
    println!("Ready!");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            eddn.subscribe_to_eddn().await;
        });
}
