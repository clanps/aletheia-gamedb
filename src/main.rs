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
    let mut args = std::env::args().skip(1);

    if let Some(cmd) = args.next() {
        let args: Vec<String> = args.collect();
        match cmd.as_str() {
            "backup" => commands::Backup::run(args, &config),
            "restore" => commands::Restore::run(args, &config),
            "update" => commands::Update::run(args, &config),
            _ => eprintln!("Command not found.")
        }
    } else {
        ui::run(&config);
    }
}
