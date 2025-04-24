// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::{create_dir_all, read_to_string, write};
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    pub save_dir: PathBuf,
}

impl Config {
    pub fn load() -> Self {
        let dir = crate::dirs::config().join("aletheia");
        let config_path = dir.join("config.json");

        if Path::exists(&config_path) {
            let content = read_to_string(&config_path).expect("Failed to read config file.");
            let cfg: Self = serde_json::from_str(&content).expect("Failed to parse config file.");

            return cfg;
        }

        let default = Self::default();

        create_dir_all(&dir).unwrap();
        write(&config_path, serde_json::to_string_pretty(&default).unwrap()).unwrap();

        default
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: if cfg!(unix) {
                std::env::var_os("XDG_DATA_HOME")
                    .map_or_else(|| std::env::var_os("HOME")
                    .map(PathBuf::from).unwrap()
                    .join(".local/share"), PathBuf::from)
                    .join("aletheia")
            } else {
                std::env::var_os("LOCALAPPDATA")
                    .map(PathBuf::from)
                    .unwrap()
                    .join("aletheia")
            }
        }
    }
}
