pub mod backup;

pub trait Command {
    fn run(args: std::env::Args, config: &crate::config::Config);
}
