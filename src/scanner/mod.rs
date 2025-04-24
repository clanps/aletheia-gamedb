// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

pub mod lutris;

#[derive(Clone, Debug)]
pub struct Game {
    pub name: String,
    pub directory: std::path::PathBuf
}

pub trait Scanner {
    fn get_games() -> anyhow::Result<Vec<Game>>;
}
