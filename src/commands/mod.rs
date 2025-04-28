// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

pub mod backup;
pub mod restore;
pub mod update;

pub trait Command {
    fn run(args: Vec<String>, config: &crate::config::Config);
}
