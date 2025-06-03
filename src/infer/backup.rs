use crate::config::Config;
use crate::gamedb;
use crate::infer::Launcher;
use crate::infer::launchers::Lutris;
use crate::operations::backup_game;

pub fn backup(launcher: &str, config: &Config) {
    let game = match launcher {
        "heroic" => todo!("Support Heroic games"),
        "lutris" => Lutris::get_game(),
        _ => {
            log::warn!("Backup was ran with infer using an unsupported launcher.");
            return;
        }
    };

    let game_db = gamedb::parse();

    if let Some(game) = game {
        if let Err(e) = backup_game(&game, config, game_db.get(&game.name).unwrap()) {
            log::error!("Failed to backup {}: {}", game.name, e);
        } else {
            log::info!("Backed up {}.", game.name);
        }
    }
}
