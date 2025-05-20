// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::{expand_path, shrink_path};
use crate::file::hash_file;
use crate::gamedb::{self, GameDbEntry, GameInfo, FileMetadata};
use crate::scanner::Game;
use super::{Args, Command};
use std::fs::{copy, create_dir_all, metadata, read_to_string, write};
use std::path::PathBuf;
use glob::glob;

pub struct Backup;

impl Command for Backup {
    fn run(args: Args, config: &Config) {
        let game_db = gamedb::parse();
        let installed_games = gamedb::get_installed_games();

        if let Some(launcher) = args.get_flag_value("infer") {
            if launcher != "lutris" {
                println!("Unsupported launcher, currently only Lutris is supported.");
                return;
            }

            let Ok(game_name) = std::env::var("GAME_NAME") else {
                println!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
                return;
            };

            if let Some(game) = installed_games.into_iter().find(|game| game.name == game_name) {
                backup_game(&game, config, game_db.get(&game.name).unwrap());
            }
        } else if !args.positional.is_empty() {
            installed_games.iter()
                .filter(|game| args.positional.contains(&game.name))
                .for_each(|game| backup_game(game, config, game_db.get(&game.name).unwrap()));
        } else {
            installed_games.iter().for_each(|game| backup_game(game, config, game_db.get(&game.name).unwrap()));
        }
    }
}

fn backup_game(game: &Game, config: &Config, entry: &GameDbEntry) {
    let backup_folder = PathBuf::from(&config.save_dir).join(game.name.replace(':', "")); // NTFS doesn't support : and this makes sense on Unix for cross-OS syncing
    let manifest_path = backup_folder.join("aletheia_manifest.yaml");
    let mut changed = false;
    let existing_manifest = manifest_path.exists().then(|| {
        let content = read_to_string(&manifest_path).unwrap();
        serde_yaml::from_str::<GameInfo>(&content).unwrap()
    });

    create_dir_all(&backup_folder).unwrap_or_else(|_| panic!("Failed to backup {}.", game.name)); // TODO: Show warning?

    let mut game_files: Vec<FileMetadata> = vec![];

    if let Some(ref windows_paths) = entry.files.windows {
        for path in windows_paths {
            let expanded = expand_path(path, game.installation_dir.as_ref(), game.prefix.as_ref());
            let found_paths = glob(&expanded.to_string_lossy()).unwrap();

            for file in found_paths {
                let file = file.unwrap();
                let file_path_str = file.to_string_lossy().to_string();

                if file.is_dir() {
                    println!("Found {file_path_str} while backing up {}. Glob patterns should match files only.", game.name);
                    continue;
                }

                let file_hash = hash_file(&file);
                let file_changed = existing_manifest.as_ref().is_none_or(|manifest| manifest.files.iter().find(|m| m.path == file_path_str).is_none_or(|metadata| metadata.hash != file_hash));

                if file_changed {
                    let file_metadata = process_file(
                        &file,
                        &backup_folder.clone().join(file.file_name().unwrap()),
                        game
                    );

                    game_files.push(file_metadata);
                    changed = true;
                } else {
                    let existing_metadata = existing_manifest.as_ref().unwrap().files
                        .iter()
                        .find(|m| m.path == file_path_str)
                        .unwrap();

                    game_files.push(existing_metadata.to_owned());
                }
            }
        }
    }

    if !changed {
        return;
    }

    let game_metadata = GameInfo {
        name: game.name.clone(),
        files: game_files
    };

    write(&manifest_path, serde_yaml::to_string(&game_metadata).unwrap()).unwrap();
    println!("Backed up {}.", game_metadata.name);
}

fn process_file(file_path: &PathBuf, dest: &PathBuf, game: &Game) -> FileMetadata {
    copy(file_path, dest).unwrap();

    FileMetadata {
        path: shrink_path(&file_path.to_string_lossy(), game.installation_dir.as_ref(), game.prefix.as_ref()).to_string_lossy().to_string(),
        hash: hash_file(file_path),
        size: metadata(file_path).unwrap().len()
    }
}
