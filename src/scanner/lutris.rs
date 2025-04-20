use std::fs::{File, read_dir};
use std::path::PathBuf;
use super::{Game, Scanner};
use anyhow::{Context, Result};

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Result<Vec<Game>> {
        let mut games = vec![];
        let lutris_config_dir = std::env::var_os("XDG_CONFIG_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".config"), PathBuf::from)
            .join("lutris")
            .join("games"); // TODO: Support Flatpak

        if !lutris_config_dir.exists() {
            return Ok(games);
        }

        let game_configs = read_dir(lutris_config_dir)?;

        for cfg in game_configs.flatten() {
            let path = cfg.path();
            
            if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
                continue;
            }

            let file = File::open(&path)?;
            let yml: serde_yaml::Value = serde_yaml::from_reader(file)?;

            let name = &yml["name"].as_str().context("Lutris game config is missing name.")?;
            let directory = &yml["game"]["prefix"].as_str().context("Lutris game config is missing game prefix.")?;

            games.push(Game { name: (*name).to_string(), directory: directory.into() });
        }

        Ok(games)
    }
}
