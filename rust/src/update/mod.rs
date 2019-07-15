use log::{debug, info};
use reqwest;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

const STORAGE_URL: &str = "https://storage.googleapis.com/ci.gs.irro.cz";

pub fn update(path: &Path) {
    if path.file_name().is_none() {
        panic!("Target path must be a file path.");
    }

    let commit = get_latest_commit();
    info!("Latest commit is {}.", commit);

    let mut download_path = PathBuf::from(path);
    download_path.set_extension("tmp");
    download_from_commit(&commit, &download_path);
    make_executable(&download_path);

    if let Err(error) = fs::rename(&download_path, path) {
        panic!(
            "Error when renaming {} to {}: {}",
            download_path.display(),
            path.display(),
            error
        );
    }
}

fn get_latest_commit() -> String {
    let latest_url = format!("{}/latest.txt", STORAGE_URL);

    let mut response = match reqwest::get(&latest_url) {
        Ok(response) => response,
        Err(error) => panic!(
            "Error when accessing file with latest commit hash: {}",
            error
        ),
    };

    let content: String = match response.text() {
        Ok(text) => text,
        Err(error) => panic!(
            "Error while downloading and decoding file with latest commit hash: {}",
            error
        ),
    };
    String::from(content.trim())
}

fn download_from_commit(commit: &str, path: &Path) {
    let binary_url = format!("{}/commits/{}/irro-cli", STORAGE_URL, commit);

    info!("Going to download {} to {}...", binary_url, path.display());

    let response = match reqwest::get(&binary_url) {
        Ok(response) => response,
        Err(error) => panic!(
            "Error when accessing binary file at {} {}",
            binary_url, error
        ),
    };

    let bytes_to_mib = |n| (n as f64) / 1_048_576.;

    let mut downloaded: u64 = 0;
    let mut downloaded_mib: f64 = 0.;
    let content_length = response.content_length();
    let content_length_mib: Option<f64> = content_length.map(bytes_to_mib);
    let mut last_downloaded_log: f64 = 0.;

    let mut reader = BufReader::new(response);
    let mut buffer: [u8; 8192] = [0; 8192];

    let target_file = match File::create(&path) {
        Err(error) => panic!("Couldn't open {}: {}", path.display(), error),
        Ok(file) => file,
    };
    let mut writer = BufWriter::new(target_file);

    loop {
        let num_bytes = match reader.read(&mut buffer) {
            Ok(0) => {
                if let Err(error) = writer.flush() {
                    panic!("Error when writing to {}: {}", path.display(), error);
                }
                break;
            }
            Ok(num_bytes) => num_bytes,
            Err(error) => panic!("Error during binary downloading: {}", error),
        };

        downloaded += u64::try_from(num_bytes).unwrap();
        downloaded_mib = bytes_to_mib(downloaded);

        if let Some(content_length_mib) = content_length_mib {
            if last_downloaded_log < (downloaded_mib - 1.) {
                last_downloaded_log = downloaded_mib;
                debug!(
                    "Downloaded {:.2} MiB of {:.2} Mib.",
                    downloaded_mib, content_length_mib
                );
            }
        }

        let mut data = Vec::with_capacity(num_bytes);
        data.extend_from_slice(&buffer[..num_bytes]);
        if let Err(error) = writer.write_all(&data) {
            panic!("Error when writing to {}: {}", path.display(), error);
        }
    }

    info!("Successfully downloaded {:.2} MiB.", downloaded_mib);
}

fn make_executable(path: &Path) {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => panic!(
            "Could not open {} for permission update: {}",
            path.display(),
            error
        ),
    };
    let metadata = match file.metadata() {
        Ok(metadata) => metadata,
        Err(error) => panic!(
            "Could retrieve file metadata of {}: {}",
            path.display(),
            error
        ),
    };
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o744);
    if let Err(error) = fs::set_permissions(path, permissions) {
        panic!("Could not set permissions to {}: {}", path.display(), error);
    }
}
