// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::updater;
use super::{Args, Command};

pub struct Update;

impl Command for Update {
    fn run(_args: Args, _config: &Config) {
        match updater::check() {
            Ok(updater::UpdateStatus::Available(_)) => println!("Aletheia is out of date! You can download the newest release here: https://github.com/Spencer-0003/aletheia/releases/latest"),
            Ok(updater::UpdateStatus::UpToDate) => println!("Aletheia is already up to date."),
            Err(e) => eprintln!("Error checking for updates: {e}")
        }
    }
}

