// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::expand_path;
use crate::file::hash_file;
use super::{Args, Command};
use std::path::{Path, PathBuf};

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
                println!("Unsupported launcher, currently only Lutris is supported.");
                return;
            }

            let Ok(game_name) = std::env::var("GAME_NAME") else {
                println!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
                return;
            };

            let game_dir = save_dir.join(&game_name);
            if !game_dir.exists() || !game_dir.is_dir() {
                println!("No backups found for {game_name}.");
                return;
            }

            let manifest_path = game_dir.join("aletheia_manifest.yaml");

            if !manifest_path.exists() {
                println!("{game_name} is missing a manifest file.");
                return;
            }

            let manifest_content = std::fs::read_to_string(manifest_path).unwrap();
            let Ok(manifest) = serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) else {
                eprintln!("Failed to parse {}'s manifest.", game_dir.file_name().unwrap().to_string_lossy());
                return;
            };

            restore_game(&game_dir, manifest, &installed_games);
            return;
        }

        let games = if args.positional.is_empty() {
            vec![]
        } else {
            args.positional
        };

        for game in std::fs::read_dir(&save_dir).unwrap() {
            let game_dir = game.unwrap().path();
            let is_dir = game_dir.is_dir();
            let game_name = game_dir.file_name().unwrap().to_string_lossy();

            if !is_dir || (is_dir && game_dir.starts_with(".")) {
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
                return;
            };

            if !games.is_empty() && !games.contains(&manifest.name) {
                continue;
            }

            restore_game(&game_dir, manifest, &installed_games);
        }
    }
}

fn restore_game(game_dir: &Path, manifest: crate::gamedb::GameInfo, lutris_games: &[crate::scanner::Game]) {
    let game_name = manifest.name;

    let Some(game) = lutris_games.iter().find(|g| g.name == game_name) else {
        println!("{game_name} was not found in Lutris.");
        return;
    };

    let mut restored = false;

    for file in &manifest.files {
        let src_file = game_dir.join(PathBuf::from(&file.path).file_name().unwrap());

        if !src_file.exists() || hash_file(&src_file) != file.hash {
            eprintln!("{} is missing or corrupted.", src_file.file_name().unwrap().to_string_lossy());
            return;
        }
    }

    for file in manifest.files {
        let expanded = expand_path(&file.path, Some(&game.directory));
        let src_file = game_dir.join(PathBuf::from(&file.path).file_name().unwrap());

        if expanded.exists() && hash_file(&expanded) == file.hash {
            continue;
        }

        let expanded_parent = expanded.parent().unwrap();
        if !&expanded_parent.exists() {
            std::fs::create_dir_all(expanded_parent).unwrap();
        }

        std::fs::copy(&src_file, &expanded).unwrap();
        restored = true;
    }

    if restored {
        println!("Restored {game_name}.");
    }
}
