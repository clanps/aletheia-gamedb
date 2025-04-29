// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use sha2::{Sha512, Digest};

pub fn hash_file(file_path: &std::path::PathBuf) -> String {
    println!("{file_path:?}");
    let mut file_content = std::fs::File::open(file_path).unwrap();
    let mut hasher = Sha512::new();

    std::io::copy(&mut file_content, &mut hasher).unwrap();

    format!("{:x}", hasher.finalize())
}
