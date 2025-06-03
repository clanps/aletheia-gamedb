// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod heroic;

#[cfg(unix)]
mod lutris;

pub use heroic::Heroic;

#[cfg(unix)]
pub use lutris::Lutris;
