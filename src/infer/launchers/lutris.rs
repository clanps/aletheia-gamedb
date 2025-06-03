use crate::gamedb;
use crate::infer::Launcher;
use crate::scanner::Game;

pub struct Lutris;

impl Launcher for Lutris {
    fn get_game() -> Option<Game> {
        let Ok(game_name) = std::env::var("GAME_NAME") else {
            log::error!("GAME_NAME environment variable not found, is the game being launched by Lutris?");
            return None;
        };

        gamedb::get_installed_games().into_iter().find(|game| game.name == game_name)
    }
}
