// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use serde::Deserialize;
use steamlocate::SteamDir;
use super::{Game, Scanner};
use std::collections::HashMap;
use std::fs::File;

#[derive(Deserialize, Debug)]
struct LoginUsersFile {
    #[serde(flatten)]
    users: HashMap<String, LoginUser>
}

#[derive(Deserialize, Debug)]
pub struct LoginUser {
    #[serde(rename = "PersonaName")]
    pub persona_name: String
}

pub struct SteamScanner;

impl SteamScanner {
    pub const fn id64_to_id3(id64: u64) -> u64 {
        id64 - 76561197960265728
    }

    pub fn get_users() -> Option<HashMap<String, LoginUser>> {
        let Ok(steam_directory) = SteamDir::locate() else {
            return None;
        };

        let login_users: LoginUsersFile = keyvalues_serde::from_reader(File::open(steam_directory.path().join("config/loginusers.vdf")).unwrap()).unwrap();

        Some(login_users.users)
    }
}

impl Scanner for SteamScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let Ok(steam_directory) = SteamDir::locate() else {
            return games;
        };

        let Ok(libraries) = steam_directory.libraries() else {
            return games; // This can fail if Steam is downloaded but never signed in
        };

        for library in libraries {
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

