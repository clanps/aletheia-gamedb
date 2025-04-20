use crate::dirs::expand_path;
use crate::scanner::lutris::LutrisScanner;
use crate::scanner::Scanner;
use super::Command;
use std::fs::{copy, create_dir_all};
use std::path::PathBuf;
use glob::glob;

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

            if let Some(windows_paths) = &game_entry.files.windows {
                for path in windows_paths {
                    let expanded = expand_path(path, Some(&game.directory));
                    let found_paths = glob(&expanded.to_string_lossy()).unwrap();

                    for file in found_paths {
                        let file_path = file.unwrap();
                        copy(&file_path, backup_folder.clone().join(file_path.file_name().unwrap())).unwrap();
                    }
                }
            }
        }
    }
}
