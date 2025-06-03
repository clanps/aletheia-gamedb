// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs::expand_path;
use crate::file::hash_file;
use std::path::{Path, PathBuf};

pub fn restore_game(game_dir: &Path, manifest: crate::gamedb::GameInfo, installed_games: &[crate::scanner::Game]) {
    let game_name = manifest.name;

    let Some(game) = installed_games.iter().find(|g| g.name == game_name) else {
        println!("{game_name} was not found.");
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
        let expanded = expand_path(&file.path, game.installation_dir.as_ref(), game.prefix.as_ref());
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

