use std::fs::{remove_dir_all, File};
use std::{io, path::Path};
use async_std::task;

use sha2::{digest::Digest, Sha256};

use rzip::unzip_archive;

#[test]
fn test_unzip() {
    task::block_on(async {
        let _ = unzip_archive(
            File::open("tests/dummy_files/zip_10MB.zip").expect("The dummy zip file was not found"),
            Path::new("tests/temp"),
        )
        .await;

        let files = vec![
            (
                "tests/temp/zip_10MB/file-example_PDF_1MB.pdf",
                "5E4D40FCD8B22453A5DA2D32533B128F2565F3FC7A4D1647A93C86CDBB4BE37A",
            ),
            (
                "tests/temp/zip_10MB/file_example_JPG_1MB.jpg",
                "683A8528125CA09D8314435C051331DE2B4C981C756721A2D12C103E8603A1D2",
            ),
            (
                "tests/temp/zip_10MB/file_example_ODS_5000.ods",
                "2DD8CA392783A86AFC0B0120B333C7FF83BB5FCFB1A0088BE6D5957E05DA1C91",
            ),
            (
                "tests/temp/zip_10MB/file_example_PNG_2500kB.jpg",
                "7AB84F2D1A5806C3A0BCE9BC67FEDA52F55423950656C50B7C07733CEB6F956A",
            ),
            (
                "tests/temp/zip_10MB/file_example_PPT_1MB.ppt",
                "B709DEBB365A5437F2472F350745ED2F8A6890D7CB3D81E6750F2D5DD44625C9",
            ),
            (
                "tests/temp/zip_10MB/file_example_TIFF_10MB.tiff",
                "7D931EB0A8B51EDC594A255739785C6B297D081B27A91BF942B6475BC322596D",
            ),
            (
                "tests/temp/zip_10MB/file-sample_1MB.doc",
                "C560136E2A2B7036523F69EFDB4E9CDF369ABE167BA3A095E26D74E261774B20",
            ),
        ];

        for (file_name, expected_hash) in files {
            let hash = hash_file(file_name)
                .await
                .unwrap_or_else(|_| panic!("Couldn't hash file {}", file_name));
            assert_eq!(expected_hash, hash);
        }

        remove_dir_all("tests/temp/zip_10MB").expect("The unzipped directory couldn't be deleted");
    })
}

async fn hash_file(file: &str) -> Result<String, io::Error> {
    let mut file = File::open(file)?;
    let mut hasher = Sha256::new();
    let _ = io::copy(&mut file, &mut hasher);
    Ok(format!("{:X}", hasher.result()))
}
