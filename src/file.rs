// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use sha2::{Sha512, Digest};
use std::fs::File;
use std::io::copy;
use std::path::Path;

pub fn hash_file(file_path: &Path) -> String {
    let mut file_content = File::open(file_path).unwrap();
    let mut hasher = Sha512::new();

    copy(&mut file_content, &mut hasher).unwrap();

    format!("{:x}", hasher.finalize())
}
