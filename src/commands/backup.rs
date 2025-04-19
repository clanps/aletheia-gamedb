use crate::scanner::lutris::LutrisScanner;
use crate::scanner::Scanner;
use super::Command;

pub struct Backup;

impl Command for Backup { 
    fn run(_args: std::env::Args) {
        let lutris_games = LutrisScanner::get_games();

        for game in lutris_games {
            println!("{} - {}", game.name, game.directory);
        }
    }
}
