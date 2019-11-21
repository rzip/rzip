use crate::options::CLIOptions;
use async_std::task;
use clap::{load_yaml, App};
use rzip::unzip_archive;
use std::{fs::File, path::PathBuf};
mod options;

fn main() {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let raw_options = CLIOptions::from_clap_matches(&matches);

    let options = raw_options.process_options();

    task::block_on(async {
        for filename in options.files.iter() {
            let file = File::open(filename).unwrap();
            let results =
                unzip_archive(file, &PathBuf::from(&raw_options.destination_folder)).await;

            match results {
                Err(error) => println!("Error in the archive {:?}: {:?}", filename, error),
                Ok(results) => {
                    for result in results {
                        match (result.name, result.result) {
                            (None, Err(error)) => {
                                println!("Error reading a file from archive: {:?}", error)
                            }
                            (Some(name), Err(error)) => println!(
                                "Error extracting the file {} from the archive {:?}",
                                name, error
                            ),
                            _ => {}
                        }
                    }
                }
            }
        }
    })
}
