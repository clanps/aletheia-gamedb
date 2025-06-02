// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::gamedb;
use crate::operations::backup_game;
use super::{Args, Command};

pub struct Backup;

impl Command for Backup {
    fn run(args: Args, config: &Config) {
        let game_db = gamedb::parse();
        let installed_games = gamedb::get_installed_games();

        if let Some(launcher) = args.get_flag_value("infer") {
            if launcher != "lutris" {
                println!("Unsupported launcher, currently only Lutris is supported.");
                return;
            }

            let Ok(game_name) = std::env::var("GAME_NAME") else {
                println!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
                return;
            };

            if let Some(game) = installed_games.into_iter().find(|game| game.name == game_name) {
                backup_game(&game, config, game_db.get(&game.name).unwrap());
            }
        } else if !args.positional.is_empty() {
            installed_games.iter()
                .filter(|game| args.positional.contains(&game.name))
                .for_each(|game| backup_game(game, config, game_db.get(&game.name).unwrap()));
        } else {
            installed_games.iter().for_each(|game| backup_game(game, config, game_db.get(&game.name).unwrap()));
        }
    }
}

