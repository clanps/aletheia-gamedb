pub mod lutris;

#[derive(Debug)]
pub struct Game {
    pub name: String,
    pub directory: std::path::PathBuf
}

pub trait Scanner {
    fn get_games() -> anyhow::Result<Vec<Game>>;
}
