use crate::dirs::{config, app_data};
use std::fs::{File, read_dir};
use super::{Game, Scanner};
use anyhow::{Context, Result};

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Result<Vec<Game>> {
        let mut games = vec![];
        let lutris_config_dir_deprecated = config().join("lutris/games");
        let lutris_config_dir_new = app_data().join("lutris/games"); // TODO: Support Flatpak

        let lutris_config_dir = if lutris_config_dir_deprecated.exists() {
            lutris_config_dir_deprecated
        } else if lutris_config_dir_new.exists() {
            lutris_config_dir_new
        } else {
            return Ok(games);
        };

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
