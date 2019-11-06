use std::{fs::{File}, io, path::{Path, PathBuf}};
use async_std::{fs::{create_dir_all, set_permissions, Permissions}};
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
    if cfg!(unix) {
        set_unix_permissions(&outpath, directory, &file).await;
    }

    Ok(())
}

#[cfg(unix)]
async fn set_unix_permissions(outpath: &PathBuf, directory: &Path, file: &ZipFile<'_>) {
    use std::os::unix::fs::PermissionsExt;

    if let Some(mode) = file.unix_mode() {
        set_permissions(directory.join(&outpath), Permissions::from_mode(mode)).await.unwrap();
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
    io::copy(file.into(), &mut outfile).unwrap();
}

async fn create_folder(name: &str, outpath: &PathBuf, directory: &Path) {
    println!(
        "File {} extracted to \"{}\"",
        name,
        outpath.as_path().display()
    );
    create_dir_all(directory.join(&outpath)).await.unwrap();
}