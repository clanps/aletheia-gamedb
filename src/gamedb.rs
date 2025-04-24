// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs::cache;
use std::fs::{create_dir_all, read_to_string, write};
use std::path::Path;
use serde::{Deserialize, Serialize};

const GAMEDB_YAML: &str = include_str!("../resources/gamedb.yaml");

#[cfg(feature = "updater")]
pub enum UpdaterResult {
    Failed,
    Success,
    UpToDate
}

#[derive(Debug, Deserialize)]
pub struct GameDbEntry {
    pub files: GameFiles
}

#[derive(Debug, Deserialize)]
pub struct GameFiles {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>
}

#[derive(Deserialize, Serialize)]
pub struct GameInfo {
    pub name: String,
    pub files: Vec<FileMetadata>
}

#[derive(Clone, Deserialize, Serialize)]
pub struct FileMetadata {
    pub hash: String,
    pub path: String,
    pub size: u64
}

pub fn parse() -> std::collections::HashMap<String, GameDbEntry> {
    if cfg!(feature = "updater") {
        let gamedb_path = cache().join("gamedb.yaml");

        if Path::exists(&gamedb_path) {
            if let Ok(gamedb) = serde_yaml::from_str(&read_to_string(gamedb_path).unwrap()) {
                return gamedb;
            }

            println!("Failed to parse cached GameDB, falling back to built-in.");
        }
    }

    serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
}

#[cfg(feature = "updater")]
pub fn update() -> anyhow::Result<UpdaterResult> {
    let cache_dir = cache();

    let gamedb_path = cache_dir.join("gamedb.yaml");
    let etag_path = cache_dir.join("gamedb.etag");

    create_dir_all(cache_dir)?;

    let previous_etag = Path::exists(&etag_path).then(|| read_to_string(&etag_path).unwrap());

    let client = reqwest::blocking::Client::new();
    let mut request = client.get("https://git.usesarchbtw.lol/Spencer/aletheia/raw/branch/master/resources/gamedb.yaml");

    if let Some(ref etag) = previous_etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    let response = request.send()?;
    let status = response.status();


    if !status.is_success() {
        return Ok(UpdaterResult::Failed);
    } else if status == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(UpdaterResult::UpToDate);
    }

    let current_etag = response.headers()
        .get(reqwest::header::ETAG)
        .map(|etag| etag.to_str().unwrap().to_owned());

    write(&gamedb_path, response.bytes()?)?;
    write(&etag_path, current_etag.unwrap())?;

    Ok(UpdaterResult::Success)
}
