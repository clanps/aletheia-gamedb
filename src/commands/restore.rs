// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::expand_path;
use crate::file::hash_file;
use super::{Args, Command};
use std::path::{Path, PathBuf};

pub struct Restore;

impl Command for Restore {
    fn run(_args: Args, config: &Config) {
        let save_dir = PathBuf::from(&config.save_dir);

        if !save_dir.exists() {
            eprintln!("Backup directory doesn't exist.");
            return;
        }

        for game in std::fs::read_dir(&save_dir).unwrap() {
            let game_dir = game.unwrap().path();

            if !game_dir.is_dir() {
                continue;
            }

            let game_name = game_dir.file_name().unwrap().to_string_lossy();

            if !game_dir.join("aletheia_manifest.yaml").exists() {
                eprintln!("{game_name} is missing a manifest file.");
                continue;
            }

            restore_game(&game_dir, &game_name, &crate::gamedb::get_installed_games());
        }
    }
}

fn restore_game(game_dir: &Path, game_name: &str, lutris_games: &[crate::scanner::Game]) {
    let Some(game) = lutris_games.iter().find(|g| g.name == game_name) else {
        println!("{game_name} was not found in Lutris.");
        return;
    };

    let manifest_content = std::fs::read_to_string(game_dir.join("aletheia_manifest.yaml")).unwrap();
    let manifest = match serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) {
        Ok(manifest) => manifest,
        Err(_e) => {
            eprintln!("Failed to parse {game_name}'s manifest.");
            return;
        }
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
