// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#![warn(clippy::pedantic)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::allow_attributes_without_reason)]
#![deny(clippy::string_to_string)]
#![allow(clippy::unreadable_literal, reason = "'Readable' literals are ugly")]

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

    if let Some(cmd) = std::env::args().nth(1) {
        match cmd.as_str() {
            "backup" => commands::backup::Backup::run(vec![], &config),
            "restore" => commands::restore::Restore::run(vec![], &config),
            "update" => commands::update::Update::run(vec![], &config),
            _ => eprintln!("Command not found.")
        }
    } else {
        ui::run(&config);
    }
}
