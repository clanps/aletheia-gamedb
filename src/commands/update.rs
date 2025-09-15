// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Args, Command};
use crate::config::Config;
use crate::updater;

pub struct Update;

impl Command for Update {
    fn run(_args: Args, _config: &Config) {
        match updater::check() {
            #[rustfmt::skip]
            Ok(updater::UpdateStatus::Available(r)) => println!("Aletheia is out of date! You can download the newest release here: {}", r.url),
            Ok(updater::UpdateStatus::UpToDate) => println!("Aletheia is already up to date."),
            Err(e) => eprintln!("Error checking for updates: {e}")
        }
    }
}
