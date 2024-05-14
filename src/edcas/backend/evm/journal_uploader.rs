use crate::edcas;
use crate::edcas::backend::journal_reader::{get_journal_log_by_index, get_log_file_list};
use crate::edcas::settings::EvmSettings;
use bus::{Bus, BusReader};
use json::JsonValue;
use log::error;
use std::io::BufRead;
use std::thread;

/**
    Initializes the uploader which uploads the journal events to the edcas network, going from the latest to the oldest.
    It gives back a bus reader which gives the current index of the finished log and the total number of logs which has to be uploaded.
*/
pub fn initialize(evm_settings: &EvmSettings, journal_directory: String) -> (BusReader<i64>, i64) {
    let mut progress_bus: Bus<i64> = Bus::new(10);
    let progress_bus_reader = progress_bus.add_rx();

    let mut journal_bus: Bus<JsonValue> = Bus::new(10);
    let journal_bus_reader = journal_bus.add_rx();
    let mut evm_reader =
        edcas::backend::evm::journal_interpreter::initialize(journal_bus_reader, evm_settings);
    thread::Builder::new()
        .name("edcas-journal-uploader-evm".into())
        .spawn(move || loop {
            evm_reader.run();
        })
        .expect("Failed to create thread journal-reader-evm");

    let path = journal_directory.clone();
    let mut index: i64 = get_log_file_list(&path).len() as i64;
    thread::Builder::new()
        .name("edcas-journal-uploader".into())
        .spawn(move || {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async move {
                    while index > 0 {
                        index -= 1;
                        let mut journal = get_journal_log_by_index(path.clone(), index as usize);
                        let mut line = String::new();
                        let mut flag: usize = 1;
                        while flag != 0 {
                            match journal.read_line(&mut line) {
                                Ok(line_flag) => {
                                    if line_flag == 0 {
                                        flag = 0;
                                    } else if !line.eq("") {
                                        let json_result = json::parse(&line);
                                        match json_result {
                                            Ok(json) => {
                                                journal_bus.broadcast(json);
                                            }
                                            Err(err) => {
                                                error!("Couldn't parse json: {}", err)
                                            }
                                        }
                                    }
                                    line.clear();
                                }
                                Err(_err) => {
                                    error!("Error reading journal file!");
                                }
                            };
                        }
                        progress_bus.broadcast(index);
                    }
                })
        })
        .expect("Cannot spawn edcas-journal-uploader thread");
    (progress_bus_reader, index)
}
