// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};

pub struct SteamScanner;

impl Scanner for SteamScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let steam_directory = match steamlocate::SteamDir::locate() {
            Ok(dir) => dir,
            Err(_e) => {
                return games;
            }
        };

        for library in steam_directory.libraries().unwrap() {
            let lib = library.unwrap();

            for app in lib.apps() {
                let game = app.unwrap();
                let install_dir = steamlocate::Library::resolve_app_dir(&lib, &game);
                let game_name = game.name.unwrap();

                games.push(Game {
                    name: game_name,
                    directory: if cfg!(unix) {
                        let prefix_directory = steam_directory.path()
                            .join("steamapps/compatdata")
                            .join(game.app_id.to_string())
                            .join("pfx");

                        if prefix_directory.exists() {
                            prefix_directory
                        } else {
                            // Native Linux game
                            install_dir
                        }
                    } else {
                        install_dir
                    },
                    source: "Steam".into()
                });
            }
        }

        games
    }
}

