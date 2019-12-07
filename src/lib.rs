use async_std::fs::{create_dir_all, set_permissions, Permissions};
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};
use zip::read::ZipFile;
use zip::result::ZipError;

#[derive(Debug)]
pub struct FileUnzipResult {
    pub name: Option<String>,
    pub result: Result<(), UnzipFileError>,
}

#[derive(Debug)]
pub enum UnzipFileError {
    OtherError,
    ZipError(ZipError),
}

#[derive(Debug)]
pub enum UnzipArchiveError {
    FileReadingError(ZipError),
    ArchiveReadingError(ZipError),
}

pub async fn unzip_archive(
    file: File,
    directory: &Path,
) -> Result<Vec<FileUnzipResult>, UnzipArchiveError> {
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(result) => result,
        Err(error) => return Err(UnzipArchiveError::ArchiveReadingError(error)),
    };
    let mut results = vec![];

    for file_index in 0..archive.len() {
        match archive.by_index(file_index) {
            Ok(file) => results.push(unzip_file(file, directory).await),
            Err(error) => return Err(UnzipArchiveError::FileReadingError(error)),
        };
    }

    Ok(results)
}
// This function stills doesn't return any error, but it will once we remove the .unwrap
async fn unzip_file(mut file: ZipFile<'_>, directory: &Path) -> FileUnzipResult {
    let outpath = file.sanitized_name();
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
    set_unix_permissions(&outpath, directory, &file).await;

    FileUnzipResult {
        name: Some(file.name().to_owned()),
        //Still empty as the unzip_file function is full of unwrap and doesn't return errors
        result: Ok(()),
    }
}

#[cfg(unix)]
async fn set_unix_permissions(outpath: &PathBuf, directory: &Path, file: &ZipFile<'_>) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(mode) = file.unix_mode() {
        set_permissions(directory.join(&outpath), Permissions::from_mode(mode))
            .await
            .unwrap();
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
        Some(p) if !p.exists() => create_dir_all(directory.join(&p)).await.unwrap(),
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
    create_dir_all(directory.join(&outpath)).await.unwrap();
}
