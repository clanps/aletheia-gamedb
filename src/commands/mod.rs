// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod backup;
mod restore;
mod update;

pub use backup::Backup;
pub use restore::Restore;
pub use update::Update;

pub trait Command {
    fn run(args: Vec<String>, config: &crate::config::Config);
}
