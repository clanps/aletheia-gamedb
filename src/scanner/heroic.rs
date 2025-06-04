// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only
use crate::dirs::{app_data, config, home};
use super::{Game, Scanner};
use serde::Deserialize;
use std::fs::read_to_string;

pub struct HeroicScanner;

#[derive(Deserialize, Debug)]
struct HeroicGOGGame {
    #[serde(rename = "appName")]
    app_id: String,
    install_path: String,
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

impl Scanner for HeroicScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        let Some(heroic_path) = (if cfg!(unix) {
            [config().join("heroic"), home().join(".var/app/com.heroicgameslauncher.hgl"), home().join(".config/heroic")]
                .into_iter()
                .find(|p| p.exists())
        } else {
            let path = app_data().join("heroic");
            path.exists().then_some(path)
        }) else {
            return games;
        };

        let gog_manifest = heroic_path.join("gog_store/installed.json");

        if !gog_manifest.exists() {
            return games;
        }

        let content = read_to_string(gog_manifest).unwrap();
        let gog_manifest: HeroicGOGManifest = serde_json::from_str(&content).unwrap();

        for game in gog_manifest.installed {
            let game_data = heroic_path.join("gogdlConfig/heroic_gogdl/manifests").join(&game.app_id);

            if !game_data.exists() {
                continue;
            }

            let game_data_content = read_to_string(game_data).unwrap();
            let Ok(game_manifest) = serde_json::from_str::<HeroicGOGGameManifest>(&game_data_content) else {
                continue;
            };

            let prefix = if cfg!(unix) && game.platform == "windows" {
                let game_config = heroic_path.join("GamesConfig").join(format!("{}.json", &game.app_id));

                if !game_config.exists() {
                    continue;
                }

                let game_config_file_content = read_to_string(&game_config).unwrap();
                let Ok(game_config) = serde_json::from_str::<serde_json::Value>(&game_config_file_content) else {
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

            games.push(Game { name: game_manifest.products[0].name.clone(), installation_dir: Some(game.install_path.into()), prefix, source: "Heroic".into() });
        }

        games
    }
}
