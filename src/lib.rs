use std::{fs::{create_dir_all, File, set_permissions, Permissions}, io, path::{Path, PathBuf}};

use zip::read::ZipFile;

pub async fn unzip_archive(file: File, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(file)?;
    let mut files = vec![];

    for file_index in 0..archive.len() {
        match archive.by_index(file_index) {
            Ok(file) => files.push(unzip_file(file, directory).await),
            Err(error) => files.push(Err(Box::new(error)))
        };
    }

    Ok(())
}

async fn unzip_file<'a>(mut file: ZipFile<'a>, directory: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let outpath = file.sanitized_name();
    // We copy the name into heap in order to avoid borrowing file as we use it later.
    let name = file.name().to_owned();

    let comment = file.comment();
    if !comment.is_empty() {
        println!("File {} comment: {}", name, comment);
    }

    if file.is_dir() {
        create_folder(&name, &outpath, directory).await;
    } else {
        create_file(&name, &outpath, directory, &mut file).await;
    }

    // Get and Set permissions
    #[cfg(unix)]
        set_unix_permissions(&outpath, directory, &file);

    Ok(())
}

#[cfg(unix)]
fn set_unix_permissions(outpath: &PathBuf, directory: &Path, file: &ZipFile<'_>) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(mode) = file.unix_mode() {
        set_permissions(directory.join(&outpath), Permissions::from_mode(mode)).unwrap();
    }
}

async fn create_file(name: &str, outpath: &PathBuf, directory: &Path, file: &mut ZipFile<'_>) {
    println!(
        "File {} extracted to \"{}\" ({} bytes)",
        name,
        outpath.as_path().display(),
        file.size()
    );

    match outpath.parent() {
        Some(p) if !p.exists() => create_dir_all(directory.join(&p)).unwrap(),
        _ => {}
    }

    let mut outfile = File::create(directory.join(&outpath)).unwrap();
    io::copy(file, &mut outfile).unwrap();
}

async fn create_folder(name: &str, outpath: &PathBuf, directory: &Path) {
    println!(
        "File {} extracted to \"{}\"",
        name,
        outpath.as_path().display()
    );
    create_dir_all(directory.join(&outpath)).unwrap();
}

#[cfg(test)]
mod tests {
    use std::{io, path::Path};
    use std::fs::{File, remove_dir_all};

    use sha2::{digest::Digest, Sha256};

    use super::*;

    #[tokio::test]
    async fn test_add() {
        let _ = unzip_archive(
            File::open("tests/zip_10MB.zip").unwrap(),
            Path::new("tests"),
        ).await;

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

        remove_dir_all("tests/zip_10MB").unwrap();
    }

    fn hash_file(file: &str) -> Result<String, io::Error> {
        let mut file = File::open(file)?;
        let mut hasher = Sha256::new();
        let _ = io::copy(&mut file, &mut hasher);
        Ok(format!("{:X}", hasher.result()))
    }
}
