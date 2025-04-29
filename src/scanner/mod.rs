// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(unix)]
mod lutris;
mod steam;

#[cfg(unix)]
pub use lutris::LutrisScanner;
pub use steam::SteamScanner;

#[derive(Clone, Debug)]
pub struct Game {
    pub name: String,
    pub directory: std::path::PathBuf,
    pub source: String
}

pub trait Scanner {
    fn get_games() -> Vec<Game>;
}
