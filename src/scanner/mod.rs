// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only
use std::path::PathBuf;

#[cfg(all(unix, not(target_os = "macos")))]
mod lutris;
mod steam;

#[cfg(windows)]
mod gog;

mod heroic;

#[cfg(all(unix, not(target_os = "macos")))]
pub use lutris::LutrisScanner;
pub use steam::SteamScanner;

#[cfg(windows)]
pub use gog::GOGScanner;

pub use heroic::HeroicScanner;

#[derive(Clone, Debug)]
pub struct Game {
    pub name: String,
    pub installation_dir: Option<PathBuf>,
    #[cfg(unix)]
    pub prefix: Option<PathBuf>,
    pub source: String
}

pub trait Scanner {
    fn get_games() -> Vec<Game>;
}
