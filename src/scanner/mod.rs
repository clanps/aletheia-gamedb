pub mod lutris;

#[derive(Debug)]
pub struct Game {
    pub name: String,
    pub directory: String
}

pub trait Scanner {
    fn get_games() -> Vec<Game>;
}
