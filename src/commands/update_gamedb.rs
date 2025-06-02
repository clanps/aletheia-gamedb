// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::gamedb;
use super::{Args, Command};

pub struct UpdateGameDb;

impl Command for UpdateGameDb {
    fn run(_args: Args, _config: &Config) {
        match gamedb::update() {
            Ok(gamedb::UpdaterResult::Success) => println!("Successfully updated GameDB."),
            Ok(gamedb::UpdaterResult::UpToDate) => println!("GameDB is already up to date."),
            Err(e) => eprintln!("Error updating GameDB: {e}")
        }
    }
}
