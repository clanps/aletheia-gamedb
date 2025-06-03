// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::{create_dir_all, read_to_string, write};
use std::path::PathBuf;

#[derive(Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    pub custom_databases: Vec<String>,
    pub save_dir: PathBuf
}

impl Config {
    fn get_dir() -> PathBuf {
        if cfg!(unix) {
            crate::dirs::config().join("aletheia")
        } else {
            crate::dirs::app_data().join("aletheia")
        }
    }

    fn get_save_dir() -> PathBuf {
        if cfg!(unix) {
            crate::dirs::app_data().join("aletheia")
        } else {
            Self::get_dir().join("saves")
        }
    }

    pub fn load() -> Self {
        let dir = Self::get_dir();

        let config_path = dir.join("config.json");

        if config_path.exists() {
            let content = read_to_string(&config_path).expect("Failed to read config file.");
            let mut cfg: Self = serde_json::from_str(&content).expect("Failed to parse config file.");

            if !cfg.save_dir.exists() {
                log::warn!("Save directory does not exist, resetting.");

                cfg.save_dir = Self::get_save_dir();
                Self::save(&cfg);
            }

            return cfg;
        }

        let default = Self::default();

        create_dir_all(&dir).unwrap();
        write(&config_path, serde_json::to_string_pretty(&default).unwrap()).unwrap();

        default
    }

    pub fn save(cfg: &Self) {
        let dir = Self::get_dir();

        let config_path = dir.join("config.json");
        create_dir_all(&dir).unwrap();

        write(&config_path, serde_json::to_string_pretty(&cfg).unwrap()).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            custom_databases: vec![],
            save_dir: Self::get_save_dir()
        }
    }
}
