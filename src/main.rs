#![warn(clippy::pedantic)]

mod commands;
mod config;
mod dirs;
mod file;
mod gamedb;
mod scanner;

use commands::Command;

fn main() {
    let config = config::Config::load();

    let mut args = std::env::args();
    let cmd = args.nth(1).unwrap_or_else(|| {
        eprintln!("No command given.");
        std::process::exit(1);
    });

    match cmd.as_str() {
        "backup" => commands::backup::Backup::run(args, &config),
        "restore" => commands::restore::Restore::run(args, &config),
        #[cfg(feature = "updater")]
        "update" => commands::update::Update::run(args, &config),
        _ => eprintln!("Command not found.")
    }
}
