// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(unix)]
pub mod lutris;
pub mod steam;

#[derive(Clone, Debug)]
pub struct Game {
    pub name: String,
    pub directory: std::path::PathBuf,
    pub source: String
}

pub trait Scanner {
    fn get_games() -> anyhow::Result<Vec<Game>>;
}
