// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use super::{Game, Scanner};

pub struct SteamScanner;

impl Scanner for SteamScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let Ok(steam_directory) = steamlocate::SteamDir::locate() else {
            return games;
        };

        for library in steam_directory.libraries().unwrap() {
            let Ok(lib) = library else {
                continue; // This can fail if running in Flatpak and permissions haven't been granted
            };

            for app in lib.apps() {
                let game = app.unwrap();
                let install_dir = steamlocate::Library::resolve_app_dir(&lib, &game);
                let game_name = game.name.unwrap();

                games.push(Game {
                    name: game_name,
                    installation_dir: Some(install_dir),
                    prefix: if cfg!(unix) {
                        let prefix_directory = steam_directory.path()
                            .join("steamapps/compatdata")
                            .join(game.app_id.to_string())
                            .join("pfx");

                        prefix_directory.exists().then_some(prefix_directory)
                    } else {
                        None
                    },
                    source: "Steam".into()
                });
            }
        }

        for shortcut in steam_directory.shortcuts().unwrap() {
            let shortcut = shortcut.unwrap();

            games.push(Game {
                name: shortcut.app_name,
                installation_dir: Some(shortcut.start_dir.into()),
                prefix: if cfg!(unix) {
                    let prefix_directory = steam_directory.path()
                        .join("steamapps/compatdata")
                        .join(shortcut.app_id.to_string())
                        .join("pfx");

                    prefix_directory.exists().then_some(prefix_directory)
                } else {
                    None
                },
                source: "Steam".into()
            });
        }

        games
    }
}

