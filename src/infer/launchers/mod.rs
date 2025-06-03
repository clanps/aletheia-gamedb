// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(unix)]
mod lutris;

#[cfg(unix)]
pub use lutris::Lutris;
