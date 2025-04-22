use crate::config::Config;
use crate::dirs::expand_path;
use crate::file::hash_file;
use crate::scanner::lutris::LutrisScanner;
use crate::scanner::Scanner;
use super::Command;
use std::path::PathBuf;

pub struct Restore;

impl Command for Restore {
    fn run(_args: std::env::Args, config: &Config) {
        let lutris_games = LutrisScanner::get_games().unwrap();
        let save_dir = PathBuf::from(&config.save_dir);

        if !save_dir.exists() {
            eprintln!("Backup directory doesn't exist.");
            return;
        }

        for game in std::fs::read_dir(&save_dir).unwrap() {
            let game_dir = game.unwrap().path();

            if !game_dir.is_dir() {
                continue;
            }

            let game_name = game_dir.file_name().unwrap().to_string_lossy();

            if !game_dir.join("aletheia_manifest.yaml").exists() {
                eprintln!("{} is missing a manifest file.", game_name);
                continue;
            }

            restore_game(&game_dir, &game_name, &lutris_games);
        }
    }
}

fn restore_game(game_dir: &PathBuf, game_name: &str, lutris_games: &Vec<crate::scanner::Game>) {
    let game = match lutris_games.iter().find(|g| g.name == game_name) {
        Some(game) => game,
        None => {
            println!("{game_name} was not found in Lutris.");
            return;
        }
    };

    let manifest_content = std::fs::read_to_string(&game_dir.join("aletheia_manifest.yaml")).unwrap();
    let manifest = match serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) {
        Ok(manifest) => manifest,
        Err(_e) => {
            eprintln!("Failed to parse {game_name}'s manifest.");
            return;
        }
    };

    let mut restored = false;

    for file in manifest.files {
        let expanded = expand_path(&file.path, Some(&game.directory));
        let src_file = game_dir.join(PathBuf::from(&file.path).file_name().unwrap());
        
        if !src_file.exists() || hash_file(&src_file) != file.hash {
            eprintln!("{} is missing or corrupted.", src_file.file_name().unwrap().to_string_lossy());
            continue;
        }

        if expanded.exists() && hash_file(&expanded) == file.hash {
            continue;
        }

        std::fs::copy(&src_file, &expanded).unwrap();
        restored = true;
    }

    if restored {
        println!("Restored {game_name}.");
    }
}
