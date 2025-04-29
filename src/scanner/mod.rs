// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(unix)]
mod lutris;
mod steam;

#[cfg(unix)]
pub use lutris::LutrisScanner;
pub use steam::SteamScanner;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{game_name} is missing {key}")]
    MissingMetadata { game_name: String, key: String },
    #[error("YAML Error: {0}")]
    MalformedYaml(#[from] serde_yaml::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub struct Game {
    pub name: String,
    pub directory: std::path::PathBuf,
    pub source: String
}

pub trait Scanner {
    fn get_games() -> Result<Vec<Game>>;
}
