// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs::{config, app_data, home};
use std::fs::{File, read_dir};
use super::{Game, Scanner};

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];
        let lutris_config_dir_deprecated = config().join("lutris/games");
        let lutris_config_dir_new = app_data().join("lutris/games");
        let lutris_config_dir_flatpak = home().join(".var/app/net.lutris.Lutris/data/lutris/games");

        let lutris_config_dir = if lutris_config_dir_deprecated.exists() {
            lutris_config_dir_deprecated
        } else if lutris_config_dir_new.exists() {
            lutris_config_dir_new
        } else if lutris_config_dir_flatpak.exists() {
            lutris_config_dir_flatpak
        } else {
            return games;
        };

        let game_configs = read_dir(lutris_config_dir).unwrap();

        for cfg in game_configs.flatten() {
            let path = cfg.path();

            if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
                continue;
            }

            let file = File::open(&path).unwrap();

            let yml: serde_yaml::Value = match serde_yaml::from_reader(file) {
                Ok(y) => y,
                Err(_) => continue
            };

            let name = yml.get("name")
                .and_then(|n| n.as_str());

            let directory = yml.get("game")
                .and_then(|g| g.get("prefix"))
                .and_then(|p| p.as_str());

            if name.is_none() || directory.is_none() {
                continue;
            }

            games.push(Game { name: name.unwrap().to_owned(), directory: directory.unwrap().into(), source: "Lutris".to_owned() });
        }

        games
    }
}
