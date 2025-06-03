// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::gamedb;
use crate::infer::Launcher;
use crate::infer::launchers::Heroic;
use crate::operations::backup_game;

#[cfg(unix)]
use crate::infer::launchers::Lutris;

pub fn backup(launcher: &str, config: &Config) {
    println!("{}", launcher.to_lowercase());
    let game = match launcher.to_lowercase().as_str() {
        "heroic" => Heroic::get_game(),
        #[cfg(unix)]
        "lutris" => Lutris::get_game(),
        _ => {
            log::warn!("Backup was ran with infer using an unsupported launcher.");
            return;
        }
    };

    let game_db = gamedb::parse();

    if let Some(game) = game {
        if let Err(e) = backup_game(&game, config, game_db.get(&game.name).unwrap()) {
            log::error!("Failed to backup {}: {}", game.name, e);
        } else {
            log::info!("Backed up {}.", game.name);
        }
    }
}
