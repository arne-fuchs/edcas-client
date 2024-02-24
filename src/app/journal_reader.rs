use std::fs::{DirEntry, File};
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use std::{fs, process};

use bus::Bus;
use chrono::NaiveDateTime;
use json::JsonValue;
use log::{debug, error, info};

use crate::app::settings::{ActionAtShutdownSignal, Settings};

pub struct JournalReader {
    pub reader: BufReader<File>,
    index: usize,
    settings: Arc<Settings>,
}

pub fn initialize(settings: Arc<Settings>) -> JournalReader {
    let mut reader = get_journal_log_by_index(
        settings.journal_reader_settings.journal_directory.clone(),
        0,
    );
    let flag = 1;
    if flag != 0 {
        loop {
            //while flag != 0 {
            let mut line = String::new();

            match reader.read_line(&mut line) {
                Ok(flag) => {
                    if flag == 0 {
                        break;
                    } else if !line.eq("") {
                        let json_result = json::parse(&line);
                        match json_result {
                            Ok(json) => {
                                let event = json["event"].as_str().unwrap();
                                if event == "Shutdown" {
                                    //debug!("\n\nReached Shutdown -> looking for newer journals\n");
                                    sleep(Duration::from_secs(1));
                                    reader = get_journal_log_by_index(
                                        settings.journal_reader_settings.journal_directory.clone(),
                                        0,
                                    )
                                }
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
    }

    info!("Journal reader found new journal -> initializing");

    JournalReader {
        reader: get_journal_log_by_index(
            settings.journal_reader_settings.journal_directory.clone(),
            0,
        ),
        index: 0,
        settings,
    }
}

impl JournalReader {
    pub fn run(&mut self, journal_bus: &mut Bus<JsonValue>) {
        let mut line = String::new();

        match self.reader.read_line(&mut line) {
            Ok(flag) => {
                if flag == 0 {
                    //TODO Detect when file is ending ( has to detect "crashes" )
                    //Reached EOF -> does not mean new data wont come in
                    //debug!("\n\nReached EOF -> increasing index and reading older journals\n");
                    //self.index = self.index + 1;
                    //self.reader = get_journal_log_by_index(self.directory_path.clone(),self.index.clone())
                } else if !line.eq("") {
                    let json_result = json::parse(&line);
                    match json_result {
                        Ok(json) => {
                            let event = json["event"].as_str().unwrap();
                            if event == "Shutdown" {
                                match self
                                    .settings
                                    .journal_reader_settings
                                    .action_at_shutdown_signal
                                {
                                    ActionAtShutdownSignal::Exit => {
                                        process::exit(0);
                                    }
                                    ActionAtShutdownSignal::Nothing => {}
                                    ActionAtShutdownSignal::Continue => {
                                        debug!("\n\nReached Shutdown -> increasing index and reading older journals\n");
                                        self.index += 1;
                                        self.reader = get_journal_log_by_index(
                                            self.settings
                                                .journal_reader_settings
                                                .journal_directory
                                                .clone(),
                                            self.index,
                                        )
                                    }
                                }
                            }
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
}

fn get_journal_log_by_index(mut directory_path: String, index: usize) -> BufReader<File> {
    let directory = fs::read_dir(directory_path.clone()).unwrap();

    let mut log_name_date_list: Vec<String> = Vec::new();

    for file in directory {
        let dir_entry: DirEntry = file.unwrap();
        let file_name: String = dir_entry.file_name().into_string().unwrap().to_owned();
        let split_file_name = file_name.split('.');
        let name_parts: Vec<&str> = split_file_name.collect::<Vec<&str>>();

        if name_parts[&name_parts.len() - 1] == "log" {
            log_name_date_list.push(name_parts[1].to_owned());
        }
    }

    let date_string_format = "%Y-%m-%dT%H%M%S";
    log_name_date_list.sort_by(|a, b| {
        let date_time_a = NaiveDateTime::parse_from_str(a, date_string_format).unwrap_or_default();
        let date_time_b = NaiveDateTime::parse_from_str(b, date_string_format).unwrap_or_default();

        date_time_a.cmp(&date_time_b).reverse()
    });

    //Reader WILL crash at this point by an index out of bounds exception if it cant find any more logs
    directory_path.push_str("/Journal.");
    directory_path.push_str(log_name_date_list[index].to_owned().as_str());
    directory_path.push_str(".01.log");

    let journal_log_file = File::open(&directory_path).expect("file not found!");

    BufReader::new(journal_log_file)
}
