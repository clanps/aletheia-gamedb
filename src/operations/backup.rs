// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::{expand_path, shrink_path};
use crate::file::hash_file;
use crate::gamedb::{GameDbEntry, GameInfo, FileMetadata};
use crate::scanner::Game;
use std::fs::{copy, create_dir_all, metadata, read_to_string, write};
use std::path::{Path, PathBuf};
use glob::glob;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create backup directory: {0}")]
    DirectoryCreation(#[from] std::io::Error),
    #[error("Malformed manifest file")]
    MalformedManifest
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn backup_game(game: &Game, config: &Config, entry: &GameDbEntry) -> Result<bool> {
    let backup_folder = PathBuf::from(&config.save_dir).join(game.name.replace(':', "")); // NTFS doesn't support : and this makes sense on Unix for cross-OS syncing
    let manifest_path = backup_folder.join("aletheia_manifest.yaml");
    let existing_manifest = manifest_path.exists()
        .then(|| read_to_string(&manifest_path).unwrap())
        .map(|content| serde_yaml::from_str::<GameInfo>(&content).map_err(|_| Error::MalformedManifest))
        .transpose()?;

    let mut changed = false;
    let mut game_files: Vec<FileMetadata> = vec![];
    let mut paths = vec![];

    if let Some(ref windows_paths) = entry.files.windows {
        paths.extend(windows_paths);
    }

    #[cfg(unix)]
    if let Some(ref linux_paths) = entry.files.linux {
        paths.extend(linux_paths);
    }

    let mut files = vec![];

    for path in paths {
        let expanded = expand_path(Path::new(path), game.installation_dir.as_ref(), game.prefix.as_ref());
        let found_paths = glob(&expanded.to_string_lossy()).unwrap();

        for file in found_paths {
            let file = file.unwrap();

            if file.is_dir() {
                log::warn!("Found {} while backing up {}. Glob patterns should match files only.", file.display(), game.name);
                continue;
            }

            files.push(file);
        }
    }

    if files.is_empty() {
        return Ok(false);
    }

    create_dir_all(&backup_folder)?;

    for file in files {
        let shrunk_file_path = shrink_path(file.as_path(), game.installation_dir.as_ref(), game.prefix.as_ref()).to_string_lossy().to_string();
        let should_backup = existing_manifest.as_ref().is_none_or(|manifest| {
            manifest.files.iter()
                .find(|m| m.path == shrunk_file_path).is_none_or(|existing| {
                    existing.hash != hash_file(&file) && metadata(&file).unwrap().modified().unwrap() > existing.modified
                })
        });

        if should_backup {
            let file_metadata = process_file(
                &file,
                &backup_folder.join(file.file_name().unwrap()),
                game
            );

            game_files.push(file_metadata);
            changed = true;
        } else {
            let existing_metadata = existing_manifest.as_ref().unwrap().files
                .iter()
                .find(|m| m.path == shrunk_file_path)
                .unwrap();

            game_files.push(existing_metadata.to_owned());
        }
    }

    if !changed {
        return Ok(false);
    }

    let game_metadata = GameInfo {
        name: game.name.clone(),
        files: game_files
    };

    write(&manifest_path, serde_yaml::to_string(&game_metadata).unwrap()).unwrap();

    Ok(true)
}

fn process_file(file_path: &PathBuf, dest: &PathBuf, game: &Game) -> FileMetadata {
    copy(file_path, dest).unwrap();

    let file_metadata = metadata(file_path).unwrap();

    FileMetadata {
        modified: file_metadata.modified().unwrap(),
        path: shrink_path(file_path, game.installation_dir.as_ref(), game.prefix.as_ref()).to_string_lossy().to_string(),
        hash: hash_file(file_path),
        size: file_metadata.len()
    }
}

