#![warn(clippy::pedantic)]

mod config;
mod scanner;

use scanner::Scanner;

fn main() {
    config::Config::load();

    let lutris_games = scanner::lutris::LutrisScanner::get_games();
    println!("Lutris games: {lutris_games:?}");
}
