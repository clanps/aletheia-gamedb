use crate::config::Config;
use crate::gamedb;
use super::Command;

pub struct Update;

impl Command for Update {
    fn run(_args: std::env::Args, _config: &Config) {
        match gamedb::update().unwrap() {
            gamedb::UpdaterResult::Success => println!("Successfully updated GameDB."),
            gamedb::UpdaterResult::Failed => println!("Failed to update GameDB."),
            gamedb::UpdaterResult::UpToDate => println!("GameDB is already up to date.")
        }
    }
}
