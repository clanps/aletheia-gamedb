use crate::config::Config;
use crate::gamedb;
use crate::infer::Launcher;
use crate::operations::restore_game;
use std::fs::read_to_string;
use std::path::PathBuf;

#[cfg(unix)]
use crate::infer::launchers::Lutris;

pub fn restore(launcher: &str, config: &Config) {
    let game = match launcher {
        "heroic" => todo!("Support Heroic games"),
        #[cfg(unix)]
        "lutris" => Lutris::get_game(),
        _ => {
            log::warn!("Backup was ran with infer using an unsupported launcher.");
            return;
        }
    };

    let save_dir = PathBuf::from(&config.save_dir);

    if let Some(game) = game {
        let game_dir = save_dir.join(&game.name);

        if !game_dir.exists() || !game_dir.is_dir() {
            log::warn!("No backups found for {}.", game.name);
            return;
        }

        let manifest_path = game_dir.join("aletheia_manifest.yaml");

        if !manifest_path.exists() {
            log::error!("{} is missing a manifest file.", game.name);
            return;
        }

        let manifest_content = read_to_string(manifest_path).unwrap();
        let Ok(manifest) = serde_yaml::from_str::<crate::gamedb::GameInfo>(&manifest_content) else {
            log::error!("Failed to parse {}'s manifest.", game_dir.file_name().unwrap().to_string_lossy());
            return;
        };

        if let Err(e) = restore_game(&game_dir, &manifest, &gamedb::get_installed_games()) {
            log::error!("Failed to restore {}: {}", game.name, e);
        } else {
            log::info!("Restore up {}.", game.name);
        }
    }
}

