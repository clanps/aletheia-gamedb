// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

mod heroic;

#[cfg(all(unix, not(target_os = "macos")))]
mod lutris;

pub use heroic::Heroic;

#[cfg(all(unix, not(target_os = "macos")))]
pub use lutris::Lutris;
