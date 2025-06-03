mod backup;
mod launchers;
mod restore;

pub use backup::backup;
pub use restore::restore;

pub trait Launcher {
    fn get_game() -> Option<crate::scanner::Game>;
}
