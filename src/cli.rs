use std::path::PathBuf;

use crate::journal_reader::start_bulk_upload;
use crate::settings::Settings;

/// Uploads all journal log files to the configured API server, printing progress
/// to stdout. Exits with code 1 on error, 0 on success.
pub fn upload_logs() {
    let settings = Settings::default();

    if settings.api_url.is_empty() {
        eprintln!("Error: no API URL configured. Set it in the Settings tab first.");
        std::process::exit(1);
    }

    let journal_dir = PathBuf::from(&settings.journal_reader.journal_directory);
    if !journal_dir.exists() {
        eprintln!(
            "Error: journal directory does not exist: {}",
            journal_dir.display()
        );
        std::process::exit(1);
    }

    println!("Uploading journal logs from: {}", journal_dir.display());
    println!("Server: {}", settings.api_url);

    let rx = start_bulk_upload(journal_dir, settings.api_url);

    for progress in rx {
        if progress.total_files > 0 {
            print!(
                "\rFile {}/{} — {} events uploaded",
                progress.current_file, progress.total_files, progress.lines_done
            );
            if let Some(ref err) = progress.error {
                print!(" [{}]", err);
            }
            use std::io::Write;
            let _ = std::io::stdout().flush();
        }
        if progress.done {
            println!();
            if let Some(err) = progress.error {
                eprintln!("Upload finished with errors: {}", err);
                std::process::exit(1);
            } else {
                println!("Upload complete.");
            }
            return;
        }
    }

    eprintln!("Upload thread ended unexpectedly.");
    std::process::exit(1);
}

