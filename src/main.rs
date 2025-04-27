// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#![deny(clippy::pedantic)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::allow_attributes_without_reason)]
#![deny(clippy::string_to_string)]

mod commands;
mod config;
mod dirs;
mod file;
mod gamedb;
mod scanner;
mod ui;

use commands::Command;

fn main() {
    let config = config::Config::load();

    let mut args = std::env::args();

    if let Some(cmd) = args.nth(1) {
        match cmd.as_str() {
            "backup" => commands::backup::Backup::run(args, &config),
            "restore" => commands::restore::Restore::run(args, &config),
            "update" => commands::update::Update::run(args, &config),
            _ => eprintln!("Command not found.")
        }
    } else {
        ui::run(&config);
    }
}
