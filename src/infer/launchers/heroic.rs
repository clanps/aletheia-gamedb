// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::gamedb;
use crate::infer::Launcher;
use crate::scanner::Game;

pub struct Heroic;

impl Launcher for Heroic {
    fn get_game() -> Option<Game> {
        let Ok(game_name) = std::env::var("HEROIC_GAME_TITLE") else {
            log::error!("HEROIC_GAME_TITLE environment variable not found, is the game being launched by Heroic?");
            return None;
        };

        let Ok(game_runner) = std::env::var("HEROIC_GAME_RUNNER") else {
            log::error!("HEROIC_GAME_RUNNER environment variable not found, is the game being launched by Heroic?");
            return None;
        };

        if game_runner != "gog" {
            log::warn!("Heroic infer only supports GOG games.");
            return None;
        }

        gamedb::get_installed_games().into_iter().find(|game| game.name == game_name && game.source == "Heroic")
    }
}
