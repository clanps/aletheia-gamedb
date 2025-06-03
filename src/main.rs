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
mod operations;
mod scanner;
mod ui;

#[cfg(feature = "updater")]
mod updater;

use commands::{Args, Command};

fn main() {
    env_logger::init();

    let config = config::Config::load();
    let mut args = std::env::args().skip(1);

    if let Some(cmd) = args.next() {
        let args = Args::parse(&args.collect::<Vec<String>>());
        match cmd.as_str() {
            "backup" => commands::Backup::run(args, &config),
            "restore" => commands::Restore::run(args, &config),
            #[cfg(feature = "updater")]
            "update" => commands::Update::run(args, &config),
            "update_gamedb" => commands::UpdateGameDb::run(args, &config),
            "update_custom_gamedbs" => commands::UpdateCustom::run(args, &config),
            _ => eprintln!("Command not found.")
        }
    } else {
        ui::run(&config);
    }
}
