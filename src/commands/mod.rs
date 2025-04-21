pub mod backup;
#[cfg(feature = "updater")]
pub mod update;

pub trait Command {
    fn run(args: std::env::Args, config: &crate::config::Config);
}
