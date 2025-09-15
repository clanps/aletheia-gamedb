// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Args, Command};
use crate::config::Config;
use crate::gamedb;

pub struct UpdateGameDb;

impl Command for UpdateGameDb {
    fn run(_args: Args, _config: &Config) {
        match gamedb::update() {
            Ok(true) => println!("Successfully updated GameDB."),
            Ok(false) => println!("GameDB is already up to date."),
            Err(e) => eprintln!("Error updating GameDB: {e}")
        }
    }
}
