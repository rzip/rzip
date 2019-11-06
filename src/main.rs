use crate::options::CLIOptions;
use clap::{load_yaml, App};
use rzip::unzip_archive;
use std::{fs::File, path::PathBuf};
mod options;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let raw_options = CLIOptions::from_clap_matches(&matches);

    let options = raw_options.process_options();

    for filename in options.files.iter() {
        let file = File::open(filename).unwrap();
        let _ = unzip_archive(file, &PathBuf::from(&raw_options.destination_folder)).await;
    }

    Ok(())
}
