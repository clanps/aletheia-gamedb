// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only
use super::{Game, Scanner};
use serde::Deserialize;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(all(unix, not(target_os = "macos")))]
use crate::dirs::home;

#[cfg(target_os = "macos")]
use crate::dirs::app_data;

#[cfg(not(target_os = "macos"))]
use crate::dirs::config;

pub struct HeroicScanner;

#[derive(Deserialize, Debug)]
struct HeroicGOGGame {
    #[serde(rename = "appName")]
    app_id: String,
    install_path: PathBuf,
    #[cfg(unix)]
    platform: String
}

#[derive(Deserialize, Debug)]
struct HeroicGOGProduct {
    name: String
}

#[derive(Deserialize, Debug)]
struct HeroicGOGGameManifest {
    products: Vec<HeroicGOGProduct>
}

#[derive(Deserialize, Debug)]
struct HeroicGOGManifest {
    installed: Vec<HeroicGOGGame>
}

impl HeroicScanner {
    fn get_game_name(heroic_path: &Path, game: &HeroicGOGGame) -> Option<String> {
        let manifest_path = heroic_path.join("gogdlConfig/heroic_gogdl/manifests").join(&game.app_id);

        if manifest_path.exists() {
            let Ok(manifest) = serde_json::from_reader::<File, HeroicGOGGameManifest>(File::open(manifest_path).unwrap()) else {
                return None;
            };

            return Some(manifest.products[0].name.clone());
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        {
            // Heroic doesn't store manifests for Linux games
            game.install_path
                .file_name()
                .and_then(|n| n.to_str())
                .map(ToOwned::to_owned)
        }

        #[cfg(any(windows, target_os = "macos"))]
        None
    }
}

impl Scanner for HeroicScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        #[cfg(all(unix, not(target_os = "macos")))]
        let heroic_path = [config().join("heroic"), home().join(".var/app/com.heroicgameslauncher.hgl")]
            .into_iter()
            .find(|p| p.exists());

        #[cfg(target_os = "macos")]
        let heroic_path = {
            let path = app_data().join("heroic");
            path.exists().then_some(path)
        };

        #[cfg(windows)]
        let heroic_path = {
            let path = config().join("heroic");
            path.exists().then_some(path)
        };

        let Some(heroic_path) = heroic_path else {
            return games;
        };

        let gog_manifest = heroic_path.join("gog_store/installed.json");

        if !gog_manifest.exists() {
            return games;
        }

        let Ok(gog_manifest) = serde_json::from_reader::<File, HeroicGOGManifest>(File::open(gog_manifest).unwrap()) else {
            log::error!("Failed to parse GOG manifest.");
            return games;
        };

        for game in gog_manifest.installed {
            let Some(game_name) = Self::get_game_name(&heroic_path, &game) else {
                continue;
            };

            #[cfg(unix)]
            let prefix = if game.platform == "windows" {
                let game_config = heroic_path.join("GamesConfig").join(format!("{}.json", &game.app_id));

                if !game_config.exists() {
                    continue;
                }

                let Ok(game_config) = serde_json::from_reader::<File, serde_json::Value>(File::open(game_config).unwrap()) else {
                    continue;
                };

                game_config
                    .get(&game.app_id)
                    .and_then(|c| c.get("winePrefix"))
                    .and_then(|p| p.as_str())
                    .map(Into::into)
            } else {
                None
            };

            #[cfg(unix)]
            games.push(Game { name: game_name, installation_dir: Some(game.install_path), prefix, source: "Heroic".into() });

            #[cfg(windows)]
            games.push(Game { name: game_name, installation_dir: Some(game.install_path), source: "Heroic".into() });
        }

        games
    }
}
