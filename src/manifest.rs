// TODO: Error handling

use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};
use reqwest::blocking::Client;

const MANIFEST_URL: &str = "https://raw.githubusercontent.com/mtkennerly/ludusavi-manifest/master/data/manifest.yaml"; // I'd write my own manifest saver but the PCGamingWiki API is straight up autistic.

pub fn download() {
    let cache_dir = if cfg!(unix) {
        std::env::var_os("XDG_CACHE_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".cache"), PathBuf::from)
            .join("aletheia")
    } else {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap()
            .join("aletheia")
            .join("cache")
    };

    let manifest_path = cache_dir.join("manifest.yml");
    let etag_path = cache_dir.join("manifest.etag");

    create_dir_all(cache_dir).expect("Failed to create cache directory.");

    let previous_etag = if Path::exists(&etag_path) {
        Some(read_to_string(&etag_path).unwrap())
    } else {
        None
    };

    let client = Client::new();
    let mut request = client.get(MANIFEST_URL);

    if let Some(etag) = &previous_etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    let response = request.send().unwrap();

    if response.status() == reqwest::StatusCode::NOT_MODIFIED {
        return; // Up to date
    }

    let current_etag = response.headers()
        .get(reqwest::header::ETAG)
        .map(|etag| etag.to_str().unwrap().to_string());

    let content = response.bytes().unwrap();
    write(&manifest_path, content).expect("Failed to save manifest.");

    if let Some(etag) = current_etag {
        write(&etag_path, etag).expect("Failed to save etag.");
    }
}

