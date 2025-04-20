use crate::dirs::expand_path;
use crate::scanner::lutris::LutrisScanner;
use crate::scanner::Scanner;
use super::Command;
use std::collections::HashMap;
use std::fs::{copy, create_dir_all, File, metadata, read_to_string, write};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use glob::glob;
use sha2::{Sha512, Digest};

#[derive(Deserialize, Serialize)]
struct GameInfo {
    name: String,
    files: Vec<FileMetadata>
}

#[derive(Clone, Deserialize, Serialize)]
struct FileMetadata {
    hash: String,
    path: String,
    size: u64
}

pub struct Backup;

impl Command for Backup { 
    fn run(_args: std::env::Args) {
        let game_db = crate::gamedb::parse();
        let lutris_games = LutrisScanner::get_games();

        for game in lutris_games {
            if !game_db.contains_key(&game.name) {
                println!("Skipping {}: Not found in GameDB", game.name);
                continue;
            }

            let backup_folder = PathBuf::from(format!("backups/{}", game.name));
            let manifest_path = backup_folder.join("aletheia_manifest.yaml");
            let mut changed = false;
            let existing_manifest = if manifest_path.exists() {
                let content = read_to_string(&manifest_path).unwrap();
                Some(serde_yaml::from_str::<GameInfo>(&content).unwrap())
            } else {
                None
            };

            let existing_files_map: HashMap<String, FileMetadata> = if let Some(manifest) = &existing_manifest {
                manifest.files.iter()
                    .map(|metadata| (metadata.path.clone(), metadata.clone()))
                    .collect()
            } else {
                HashMap::new()
            };

            create_dir_all(&backup_folder).expect(&format!("Failed to backup {}.", game.name));

            let game_entry = game_db.get(&game.name).unwrap();
            let mut game_files: Vec<FileMetadata> = vec![];

            if let Some(windows_paths) = &game_entry.files.windows {
                for path in windows_paths {
                    let expanded = expand_path(path, Some(&game.directory));
                    let found_paths = glob(&expanded.to_string_lossy()).unwrap();

                    for file in found_paths {
                        let file_path = file.unwrap();
                        let file_path_str = file_path.to_string_lossy().to_string();
                        let file_hash = hash_file(&file_path);

                        let file_changed = match existing_files_map.get(&file_path_str) {
                            Some(metadata) => metadata.hash != file_hash,
                            None => true
                        };

                        if file_changed {
                            let file_metadata = process_file(&file_path, &backup_folder.clone().join(file_path.file_name().unwrap()));
                            game_files.push(file_metadata);
                            changed = true;
                        } else {
                            game_files.push(existing_files_map.get(&file_path_str).unwrap().clone());
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
        }
    }
}

fn hash_file(file_path: &PathBuf) -> String {
    let mut file_content = File::open(file_path).unwrap();
    let mut hasher = Sha512::new();

    std::io::copy(&mut file_content, &mut hasher).unwrap();

    format!("{:x}", hasher.finalize())
}

fn process_file(file_path: &PathBuf, dest: &PathBuf) -> FileMetadata {
    copy(file_path, dest).unwrap();

    FileMetadata {
        path: file_path.to_string_lossy().to_string(),
        hash: hash_file(file_path),
        size: metadata(file_path).unwrap().len()
    }
}
