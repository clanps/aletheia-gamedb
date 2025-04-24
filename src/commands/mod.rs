// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

pub mod backup;
pub mod restore;
#[cfg(feature = "updater")]
pub mod update;

pub trait Command {
    fn run(args: std::env::Args, config: &crate::config::Config);
}
