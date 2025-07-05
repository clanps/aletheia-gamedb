// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs::expand_path;
use crate::file::hash_file;
use crate::gamedb::GameInfo;
use crate::scanner::Game;
use std::fs::{copy, create_dir_all};
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Game not found")]
    GameNotFound,
    #[error("{0} is missing or corrupted")]
    MissingOrCorruptedFiles(String)
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn restore_game(game_dir: &Path, manifest: &GameInfo, installed_games: &[Game]) -> Result<bool> {
    let game_name = &manifest.name;

    let Some(game) = installed_games.iter().find(|g| g.name == *game_name) else {
        return Err(Error::GameNotFound);
    };

    for file in &manifest.files {
        let src_file = game_dir.join(PathBuf::from(&file.path).file_name().unwrap());

        if !src_file.exists() || hash_file(&src_file) != file.hash {
            return Err(Error::MissingOrCorruptedFiles(src_file.file_name().unwrap().to_string_lossy().to_string()));
        }
    }

    for file in &manifest.files {
        let expanded = expand_path(Path::new(&file.path), game.installation_dir.as_deref(), game.prefix.as_deref());
        let src_file = game_dir.join(PathBuf::from(&file.path).file_name().unwrap());

        if expanded.exists() && hash_file(&expanded) == file.hash {
            continue;
        }

        let expanded_parent = expanded.parent().unwrap();
        if !&expanded_parent.exists() {
            create_dir_all(expanded_parent).unwrap();
        }

        copy(&src_file, &expanded).unwrap();
    }

    Ok(true)
}

