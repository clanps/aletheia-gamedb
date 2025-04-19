pub mod lutris;

#[derive(Debug)]
pub struct Game {
    name: String,
    directory: String
}

pub trait Scanner {
    fn get_games() -> Vec<Game>;
}
