// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::dirs::{expand_path, shrink_path};
use crate::file::hash_file;
use crate::gamedb::{self, GameInfo, FileMetadata};
use super::{Args, Command};
use std::fs::{copy, create_dir_all, metadata, read_to_string, write};
use std::path::PathBuf;
use glob::glob;

pub struct Backup;

impl Command for Backup {
    fn run(args: Args, config: &Config) {
        let game_db = gamedb::parse();
        let installed_games = gamedb::get_installed_games();
        let games: Vec<_> = if args.positional.is_empty() {
            installed_games
        } else {
            installed_games.into_iter().filter(|game| args.positional.contains(&game.name)).collect()
        };

        for game in games {
            let backup_folder = PathBuf::from(&config.save_dir).join(&game.name);
            let manifest_path = backup_folder.join("aletheia_manifest.yaml");
            let mut changed = false;
            let existing_manifest = manifest_path.exists().then(|| {
                let content = read_to_string(&manifest_path).unwrap();
                serde_yaml::from_str::<GameInfo>(&content).unwrap()
            });

            create_dir_all(&backup_folder).expect(&format!("Failed to backup {}.", game.name));

            let game_entry = game_db.get(&game.name).unwrap();
            let mut game_files: Vec<FileMetadata> = vec![];

            if let Some(ref windows_paths) = game_entry.files.windows {
                for path in windows_paths {
                    let expanded = expand_path(path, Some(&game.directory));
                    let found_paths = glob(&expanded.to_string_lossy()).unwrap();

                    for file in found_paths {
                        let file_path = file.unwrap();
                        let file_path_str = file_path.to_string_lossy().to_string();
                        let file_hash = hash_file(&file_path);
                        let file_changed = existing_manifest.as_ref().is_none_or(|manifest| manifest.files.iter().find(|m| m.path == file_path_str).is_none_or(|metadata| metadata.hash != file_hash));

                        if file_changed {
                            let file_metadata = process_file(
                                &file_path,
                                &backup_folder.clone().join(file_path.file_name().unwrap()),
                                Some(&game.directory)
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
                continue;
            }

            let game_metadata = GameInfo {
                name: game.name,
                files: game_files
            };

            write(&manifest_path, serde_yaml::to_string(&game_metadata).unwrap()).unwrap();
            println!("Backed up {}.", game_metadata.name);
        }
    }
}

fn process_file(file_path: &PathBuf, dest: &PathBuf, prefix: Option<&PathBuf>) -> FileMetadata {
    copy(file_path, dest).unwrap();

    FileMetadata {
        path: shrink_path(&file_path.to_string_lossy(), prefix).to_string_lossy().to_string(),
        hash: hash_file(file_path),
        size: metadata(file_path).unwrap().len()
    }
}
