#[macro_use]
extern crate clap;

fn main() {
    let yaml = load_yaml!("../cli_def/en_us.yml");
    let matches = clap::App::from_yaml(yaml).get_matches();

    let file = matches.value_of("files").expect("You need to provide a valid path to a compressed file.");

    let fname = std::path::Path::new(&file);
    let file = std::fs::File::open(&fname).unwrap();

    unzip_file(file, std::path::Path::new(""));
}

fn unzip_file(file: std::fs::File, directory: &std::path::Path) {
    let mut archive = zip::ZipArchive::new(file).unwrap();

    let files_index = 0..archive.len();

    files_index
    .map(|file_index| {
        unzip_archive(file_index, &mut archive, directory)
    }).for_each(drop);
}

fn unzip_archive(file_index: usize, archive: &mut zip::ZipArchive<std::fs::File>, directory: &std::path::Path) {
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
        std::fs::create_dir_all(directory.join(&outpath)).unwrap();
    } else {
        println!("File {} extracted to \"{}\" ({} bytes)", file_index, outpath.as_path().display(), file.size());
        if let Some(p) = outpath.parent() {
            if !p.exists() {
                std::fs::create_dir_all(directory.join(&p)).unwrap();
            }
        }
        let mut outfile = std::fs::File::create(directory.join(&outpath)).unwrap();
        std::io::copy(&mut file, &mut outfile).unwrap();
    }

    // Get and Set permissions
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if let Some(mode) = file.unix_mode() {
            std::fs::set_permissions(directory.join(&outpath), std::fs::Permissions::from_mode(mode)).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Sha256, digest::Digest};
    use std::{io, io::Error, path::Path, fs::File};

    #[test]
    fn test_add() {
        unzip_file(File::open("tests/zip_10MB.zip").unwrap(), Path::new("tests"));

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

    fn hash_file(file: &str) -> Result<String, Error> {
        let mut file = File::open(file)?;
        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher);
        Ok(format!("{:X}", hasher.result()))
    }
}
