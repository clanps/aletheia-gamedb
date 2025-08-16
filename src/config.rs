// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs;
use std::fs::{create_dir_all, File};
use std::path::PathBuf;

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Config {
    pub custom_databases: Vec<String>,
    pub save_dir: PathBuf,
    pub steam_account_id: Option<String>,
    #[cfg(feature = "updater")]
    pub check_for_updates: bool
}

impl Config {
    fn get_dir() -> PathBuf {
        if cfg!(unix) {
            dirs::config().join("aletheia")
        } else {
            dirs::app_data().join("aletheia")
        }
    }

    fn get_save_dir() -> PathBuf {
        if cfg!(unix) {
            dirs::app_data().join("aletheia")
        } else {
            Self::get_dir().join("saves")
        }
    }

    pub fn load() -> Self {
        let dir = Self::get_dir();
        let config_path = dir.join("config.json");

        if config_path.exists() {
            let config_file = File::open(&config_path).expect("Failed to read config file.");
            let mut cfg: Self = serde_json::from_reader(&config_file).expect("Failed to parse config file.");

            if !cfg.save_dir.exists() {
                log::warn!("Save directory does not exist, resetting.");

                cfg.save_dir = Self::get_save_dir();
                Self::save(&cfg);
            }

            return cfg;
        }

        let default = Self::default();

        create_dir_all(&dir).unwrap();
        serde_json::to_writer_pretty(File::create(&config_path).unwrap(), &default).unwrap();

        default
    }

    pub fn save(cfg: &Self) {
        let dir = Self::get_dir();
        let config_path = dir.join("config.json");

        create_dir_all(&dir).unwrap();
        serde_json::to_writer_pretty(File::create(&config_path).unwrap(), &cfg).unwrap();
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            custom_databases: vec![],
            save_dir: Self::get_save_dir(),
            steam_account_id: None,
            #[cfg(feature = "updater")]
            check_for_updates: true
        }
    }
}
