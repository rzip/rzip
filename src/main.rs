#[macro_use]
extern crate clap;

mod options;
use crate::options::CLIOptions;
use clap::App;
use std::{
    fs,
    fs::File,
    io,
    path::{Path, PathBuf},
};

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

fn unzip_file(file: File, directory: &Path) {
    let mut archive = zip::ZipArchive::new(file).unwrap();

    for file_index in 0..archive.len() {
        let file = archive.by_index(file_index).unwrap();
        unzip_archive(file, directory)
    }
}

fn unzip_archive(mut file: zip::read::ZipFile, directory: &Path) {
    let outpath = file.sanitized_name();
    let name = file.name().to_owned(); // We copy the name into heap in order to avoid borrowing file as we use it later.

    let comment = file.comment();
    if !comment.is_empty() {
        println!("File {} comment: {}", name, comment);
    }

    if file.name().ends_with('/') {
        create_folder(&name, &outpath, directory);
    } else {
        create_file(&name, &outpath, directory, &mut file)
    }

    // Get and Set permissions
    #[cfg(unix)]
    set_unix_permissions(&outpath, directory, &file);
}

#[cfg(unix)]
fn set_unix_permissions(outpath: &PathBuf, directory: &Path, file: &zip::read::ZipFile<'_>) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(mode) = file.unix_mode() {
        fs::set_permissions(directory.join(&outpath), fs::Permissions::from_mode(mode)).unwrap();
    }
}

fn create_file(name: &str, outpath: &PathBuf, directory: &Path, file: &mut zip::read::ZipFile<'_>) {
    println!(
        "File {} extracted to \"{}\" ({} bytes)",
        name,
        outpath.as_path().display(),
        file.size()
    );
    if let Some(p) = outpath.parent() {
        if !p.exists() {
            fs::create_dir_all(directory.join(&p)).unwrap();
        }
    }
    let mut outfile = File::create(directory.join(&outpath)).unwrap();
    io::copy(&mut *file, &mut outfile).unwrap();
}

fn create_folder(name: &str, outpath: &PathBuf, directory: &Path) {
    println!(
        "File {} extracted to \"{}\"",
        name,
        outpath.as_path().display()
    );
    fs::create_dir_all(directory.join(&outpath)).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{digest::Digest, Sha256};
    use std::{fs::File, io, path::Path};

    #[test]
    fn test_add() {
        unzip_file(
            File::open("tests/zip_10MB.zip").unwrap(),
            Path::new("tests"),
        );

        let hash = hash_file("tests/zip_10MB/file-example_PDF_1MB.pdf").unwrap();
        let correct_hash = "5E4D40FCD8B22453A5DA2D32533B128F2565F3FC7A4D1647A93C86CDBB4BE37A";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file_example_JPG_1MB.jpg").unwrap();
        let correct_hash = "683A8528125CA09D8314435C051331DE2B4C981C756721A2D12C103E8603A1D2";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file_example_ODS_5000.ods").unwrap();
        let correct_hash = "2DD8CA392783A86AFC0B0120B333C7FF83BB5FCFB1A0088BE6D5957E05DA1C91";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file_example_PNG_2500kB.jpg").unwrap();
        let correct_hash = "7AB84F2D1A5806C3A0BCE9BC67FEDA52F55423950656C50B7C07733CEB6F956A";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file_example_PPT_1MB.ppt").unwrap();
        let correct_hash = "B709DEBB365A5437F2472F350745ED2F8A6890D7CB3D81E6750F2D5DD44625C9";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file_example_TIFF_10MB.tiff").unwrap();
        let correct_hash = "7D931EB0A8B51EDC594A255739785C6B297D081B27A91BF942B6475BC322596D";
        assert_eq!(correct_hash, hash);

        let hash = hash_file("tests/zip_10MB/file-sample_1MB.doc").unwrap();
        let correct_hash = "C560136E2A2B7036523F69EFDB4E9CDF369ABE167BA3A095E26D74E261774B20";
        assert_eq!(correct_hash, hash);

        std::fs::remove_dir_all("tests/zip_10MB").unwrap();
    }

    fn hash_file(file: &str) -> Result<String, io::Error> {
        let mut file = File::open(file)?;
        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher);
        Ok(format!("{:X}", hasher.result()))
    }
}
