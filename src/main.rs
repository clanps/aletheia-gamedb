#![warn(clippy::pedantic)]

mod commands;
mod config;
mod manifest;
mod scanner;

use commands::Command;

fn main() {
    config::Config::load();
    manifest::download();

    let mut args = std::env::args();
    let cmd = args.nth(1).expect("No command given.");

    match cmd.as_str() {
        "backup" => commands::backup::Backup::run(args),
        _ => eprintln!("Command not found.")
    }
}
