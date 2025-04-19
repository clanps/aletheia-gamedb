#![warn(clippy::pedantic)]

mod scanner;

use scanner::Scanner;

fn main() {
    let lutris_games = scanner::lutris::LutrisScanner::get_games();
    println!("Lutris games: {lutris_games:?}");
}
