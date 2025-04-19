use std::fs::{File, read_dir};
use std::path::PathBuf;
use super::{Game, Scanner};

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];
        let lutris_config_dir = std::env::var_os("XDG_CONFIG_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".config"), PathBuf::from)
            .join("lutris")
            .join("games"); // TODO: Support Flatpak

        if !lutris_config_dir.exists() {
            return games;
        }

        let game_configs = read_dir(lutris_config_dir).expect("Failed to read Lutris game configs.");

        for cfg in game_configs.flatten() {
            let path = cfg.path();
            
            if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
                continue;
            }

            let file = File::open(&path).expect("Failed to open Lutris game config.");
            let yml: serde_yaml::Value = serde_yaml::from_reader(file).unwrap();

            let name = &yml["name"].as_str().unwrap();
            let directory = &yml["game"]["prefix"].as_str().unwrap();

            games.push(Game { name: (*name).to_string(), directory: (*directory).to_string() });
        }

        games
    }
}
