// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::gamedb;
use super::Command;

pub struct Update;

impl Command for Update {
    fn run(_args: Vec<String>, _config: &Config) {
        match gamedb::update() {
            Ok(gamedb::UpdaterResult::Success) => println!("Successfully updated GameDB."),
            Ok(gamedb::UpdaterResult::UpToDate) => println!("GameDB is already up to date."),
            Err(e) => eprintln!("Error updating GameDB: {e}")
        }
    }
}
