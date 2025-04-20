use crate::dirs::config;
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

fn expand_path(path: &str, prefix: Option<&PathBuf>) -> PathBuf {
    if cfg!(unix) {
        let wine_prefix = prefix.unwrap();
        let drive_c = wine_prefix.join("drive_c");
        let users = drive_c.join("users").join("*"); // TODO: Ignore steamuser
        let app_data = users.join("AppData");
        let documents = users.join("Documents");

        path
            .replace("{AppData}", &app_data.display().to_string())
            .replace("{Documents}", &documents.display().to_string())
            .replace("{Home}", &users.display().to_string())
            .replace("{LocalAppData}", &app_data.join("Local").display().to_string())
            .replace("{LocalLow}", &app_data.join("LocalLow").display().to_string())
            .replace("{SteamUserData}", "{SteamUserData}") // TODO
            .replace("{XDGConfig}", &config().display().to_string())
            .into()
    } else {
        todo!("Windows path expansion")
    }
}
