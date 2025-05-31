// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::cache;
use crate::scanner::{Game, Scanner};
use crate::scanner::{HeroicScanner, SteamScanner};
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, write};
use serde::{Deserialize, Serialize};

#[cfg(unix)]
use crate::scanner::LutrisScanner;

#[cfg(windows)]
use crate::scanner::GOGScanner;

const GAMEDB_YAML: &str = include_str!("../resources/gamedb.yaml");

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

pub enum UpdaterResult {
    Success,
    UpToDate
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GameDbEntry {
    pub files: GameFiles
}

#[derive(Debug, Deserialize, Serialize)]
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

pub fn parse() -> HashMap<String, GameDbEntry> {
    let cache_dir = cache();
    let custom_gamedb_path = cache_dir.join("custom_gamedb.yaml");
    let gamedb_path = cache_dir.join("gamedb.yaml");

    let mut db: HashMap<String, GameDbEntry> = if gamedb_path.exists() {
        if let Ok(gamedb) = serde_yaml::from_str(&read_to_string(gamedb_path).unwrap()) {
            gamedb
        } else {
            println!("Failed to parse cached GameDB, falling back to built-in.");
            serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
        }
    } else {
        serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
    };

    if cache_dir.exists() {
        if let Ok(gamedb) = serde_yaml::from_str::<HashMap<String, GameDbEntry>>(&read_to_string(custom_gamedb_path).unwrap()) {
            db.extend(gamedb);
        }
    }

    db
}

pub fn get_installed_games() -> Vec<Game> {
    let db = parse();
    let mut games = vec![];

    #[cfg(unix)]
    games.extend(LutrisScanner::get_games());

    #[cfg(windows)]
    games.extend(GOGScanner::get_games());

    games.extend(HeroicScanner::get_games());
    games.extend(SteamScanner::get_games());

    games.into_iter().filter(|game| db.contains_key(&game.name)).collect()
}

pub fn update() -> Result<UpdaterResult> {
    let cache_dir = cache();

    let gamedb_path = cache_dir.join("gamedb.yaml");
    let etag_path = cache_dir.join("gamedb.etag");

    create_dir_all(cache_dir)?;

    let previous_etag = etag_path.exists().then(|| read_to_string(&etag_path).ok()).flatten();

    let client = reqwest::blocking::Client::new();
    let mut request = client.get("https://git.usesarchbtw.lol/Spencer/aletheia/raw/branch/master/resources/gamedb.yaml");

    if let Some(ref etag) = previous_etag {
        request = request.header(reqwest::header::IF_NONE_MATCH, etag);
    }

    let response = request.send()?.error_for_status()?;
    let status = response.status();

    if status == reqwest::StatusCode::NOT_MODIFIED {
        return Ok(UpdaterResult::UpToDate);
    }

    let current_etag = response.headers()
        .get(reqwest::header::ETAG)
        .map(|etag| etag.to_str().unwrap().to_owned());

    write(&gamedb_path, response.bytes()?)?;
    write(&etag_path, current_etag.unwrap())?;

    Ok(UpdaterResult::Success)
}

// TODO: Make async
pub fn update_custom(cfg: &Config) -> Result<UpdaterResult> {
    if cfg.custom_databases.is_empty() {
        return Ok(UpdaterResult::UpToDate);
    }

    let cache_dir = cache();
    let client = reqwest::blocking::Client::new();
    let mut combined = HashMap::<String, GameDbEntry>::new();

    for db in &cfg.custom_databases {
        let response = client.get(db).send()?.error_for_status()?;
        let db = serde_yaml::from_str::<HashMap<String, GameDbEntry>>(&response.text()?).unwrap();

        combined.extend(db);
    }

    write(cache_dir.join("custom_gamedb.yaml"), serde_yaml::to_string(&combined).unwrap())?;

    Ok(UpdaterResult::Success)
}
