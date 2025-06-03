// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::operations::restore_game;
use super::{Args, Command};
use std::path::PathBuf;

pub struct Restore;

impl Command for Restore {
    fn run(args: Args, config: &Config) {
        let save_dir = PathBuf::from(&config.save_dir);

        if !save_dir.exists() {
            eprintln!("Backup directory doesn't exist.");
            return;
        }

        let installed_games = crate::gamedb::get_installed_games();

        if let Some(launcher) = args.get_flag_value("infer") {
            if launcher != "lutris" {
                log::warn!("Restore was ran with infer using an unsupported launcher.");
                return;
            }

            let Ok(game_name) = std::env::var("GAME_NAME") else {
                log::error!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
                return;
            };

            let game_dir = save_dir.join(&game_name);
            if !game_dir.exists() || !game_dir.is_dir() {
                log::warn!("No backups found for {game_name}.");
                return;
            }

            let manifest_path = game_dir.join("aletheia_manifest.yaml");

            if !manifest_path.exists() {
                log::error!("{game_name} is missing a manifest file.");
                return;
            }

            let manifest_content = std::fs::read_to_string(manifest_path).unwrap();
            let Ok(manifest) = serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) else {
                log::error!("Failed to parse {}'s manifest.", game_dir.file_name().unwrap().to_string_lossy());
                return;
            };

            if let Err(e) = restore_game(&game_dir, &manifest, &installed_games) {
                log::error!("Failed to restore {}: {e}", manifest.name);
            } else {
                log::info!("Restored {}.", manifest.name);
            }

            return;
        }

        for game in std::fs::read_dir(&save_dir).unwrap() {
            let game_dir = game.unwrap().path();
            let is_dir = game_dir.is_dir();
            let game_name = game_dir.file_name().unwrap().to_string_lossy();

            if !is_dir || game_name.starts_with('.') {
                continue;
            }

            let manifest_path = game_dir.join("aletheia_manifest.yaml");

            if !manifest_path.exists() {
                eprintln!("{game_name} is missing a manifest file.");
                continue;
            }

            let manifest_content = std::fs::read_to_string(manifest_path).unwrap();
            let Ok(manifest) = serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) else {
                eprintln!("Failed to parse {}'s manifest.", game_dir.file_name().unwrap().to_string_lossy());
                continue;
            };

            if !args.positional.is_empty() && !args.positional.contains(&manifest.name) {
                continue;
            }

            if let Err(e) = restore_game(&game_dir, &manifest, &installed_games) {
                println!("Failed to restore {}: {e}", manifest.name);
            } else {
                println!("Restored {}.", manifest.name);
            }
        }
    }
}
