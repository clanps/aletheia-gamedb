use crate::dirs::expand_path;
use crate::scanner::lutris::LutrisScanner;
use crate::scanner::Scanner;
use super::Command;
use std::fs::{copy, create_dir_all, File, metadata, write};
use std::path::PathBuf;
use serde::Serialize;
use glob::glob;
use sha2::{Sha512, Digest};

#[derive(Serialize)]
struct GameInfo {
    name: String,
    files: Vec<FileMetadata>
}

#[derive(Serialize)]
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
            create_dir_all(&backup_folder).expect(&format!("Failed to backup {}.", game.name));

            let game_entry = game_db.get(&game.name).unwrap();
            let mut game_files: Vec<FileMetadata> = vec![];

            if let Some(windows_paths) = &game_entry.files.windows {
                for path in windows_paths {
                    let expanded = expand_path(path, Some(&game.directory));
                    let found_paths = glob(&expanded.to_string_lossy()).unwrap();

                    for file in found_paths {
                        let file_path = file.unwrap();
                        let file_metadata = process_file(&file_path, &backup_folder.clone().join(file_path.file_name().unwrap()));

                        game_files.push(file_metadata);
                    }
                }
            }

            let game_metadata = GameInfo {
                name: game.name,
                files: game_files
            };

            write(&backup_folder.join("aletheia_manifest.yaml"), serde_yaml::to_string(&game_metadata).unwrap()).unwrap();
        }
    }
}

fn process_file(file_path: &PathBuf, dest: &PathBuf) -> FileMetadata {
    let mut file_content = File::open(file_path).unwrap();
    let mut hasher = Sha512::new();

    std::io::copy(&mut file_content, &mut hasher).unwrap();

    copy(file_path, dest).unwrap();

    FileMetadata {
        path: file_path.to_string_lossy().to_string(),
        hash: format!("{:x}", hasher.finalize()),
        size: metadata(&file_path).unwrap().len()
    }
}
