#[macro_use]
extern crate clap;

mod lib;
mod options;
use crate::lib::unzip_file;
use crate::options::CLIOptions;
use clap::App;
use std::{fs::File, path::PathBuf};

fn main() {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let raw_options = CLIOptions::from_clap_matches(&matches);

    let options = raw_options.process_options();

    for filename in options.files {
        let file = File::open(filename).unwrap();
        unzip_file(file, &PathBuf::from(&raw_options.destination_folder));
    }
}
