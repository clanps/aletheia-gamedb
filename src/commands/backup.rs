// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::gamedb;
use crate::infer;
use crate::operations::backup_game;
use super::{Args, Command};

pub struct Backup;

impl Command for Backup {
    fn run(args: Args, config: &Config) {
        let game_db = gamedb::parse();

        if let Some(launcher) = args.get_flag_value("infer") {
            infer::backup(launcher, config);
            return;
        }
        
        let installed_games = gamedb::get_installed_games();

        if args.positional.is_empty() {
            for game in &installed_games {
                if let Err(e) = backup_game(game, config, &game_db[&game.name]) {
                    eprintln!("Failed to backup {}: {}", game.name, e);
                } else {
                    println!("Backed up {}.", game.name);
                }
            }
        } else {
            installed_games.iter()
                .filter(|game| args.positional.contains(&game.name))
                .for_each(|game| {
                    if let Err(e) = backup_game(game, config, &game_db[&game.name]) {
                        eprintln!("Failed to backup {}: {}", game.name, e);
                    } else {
                        println!("Backed up {}.", game.name);
                    }
                });
        }
    }
}

