// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Args, Command};
use crate::config::Config;
use crate::gamedb;

pub struct UpdateCustom;

impl Command for UpdateCustom {
    fn run(_args: Args, config: &Config) {
        match gamedb::update_custom(config) {
            Ok(true) => println!("Successfully updated custom GameDBs."),
            Ok(false) => println!("Custom GameDBs are already up to date."),
            Err(e) => eprintln!("Error updating custom GameDBs: {e}")
        }
    }
}
