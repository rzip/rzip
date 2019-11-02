#[macro_use]
extern crate clap;

fn main() {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let file = matches.value_of("files").expect("You need to provide a valid path to a compressed file.");

    let fname = std::path::Path::new(&file);
    let file = std::fs::File::open(&fname).unwrap();

    let mut archive = zip::ZipArchive::new(file).unwrap();

    let files_index = 0..archive.len();

    files_index
    .map(|file_index| {
        let mut file = archive.by_index(file_index).unwrap();
        let outpath = file.sanitized_name();

        {
            let comment = file.comment();
            if !comment.is_empty() {
                println!("File {} comment: {}", file_index, comment);
            }
        }

        if (&*file.name()).ends_with('/') {
            println!("File {} extracted to \"{}\"", file_index, outpath.as_path().display());
            std::fs::create_dir_all(&outpath).unwrap();
        } else {
            println!("File {} extracted to \"{}\" ({} bytes)", file_index, outpath.as_path().display(), file.size());
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(&p).unwrap();
                }
            }
            let mut outfile = std::fs::File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();
        }

        // Get and Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode)).unwrap();
            }
        }
    }).for_each(drop);
}
