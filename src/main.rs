#[macro_use]
extern crate clap;

fn main() {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    clap::App::from_yaml(yaml).get_matches();
}
