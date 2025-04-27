// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::read_dir;
use std::path::PathBuf;

pub fn cache() -> PathBuf {
    if cfg!(unix) {
        std::env::var_os("XDG_CACHE_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".cache"), PathBuf::from)
            .join("aletheia")
    } else {
        config().join("aletheia/cache")
    }
}

pub fn config() -> PathBuf {
    if cfg!(unix) {
        std::env::var_os("XDG_CONFIG_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".config"), PathBuf::from)
    } else {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap()
    }
}

pub fn app_data() -> PathBuf {
    if cfg!(unix) {
        std::env::var_os("XDG_DATA_HOME")
            .map_or_else(|| std::env::var_os("HOME")
            .map(PathBuf::from).unwrap()
            .join(".local/share"), PathBuf::from)
    } else {
        std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap()
    }
}

pub fn home() -> PathBuf {
    if cfg!(unix) {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap()
    } else {
        std::env::var_os("USERPROFILE")
            .map(PathBuf::from)
            .unwrap()
    }
}

pub fn expand_path(path: &str, prefix: Option<&PathBuf>) -> PathBuf {
    if cfg!(unix) {
        let wine_prefix = prefix.unwrap();
        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(std::env::var_os("USER").unwrap());
        let windows_app_data = user.join("AppData");
        let documents = user.join("Documents");
        let linux_app_data = app_data();

        path
            .replace("{AppData}", &windows_app_data.to_string_lossy())
            .replace("{Documents}", &documents.to_string_lossy())
            .replace("{Home}", &user.to_string_lossy())
            .replace("{LocalAppData}", &windows_app_data.join("Local").to_string_lossy())
            .replace("{LocalLow}", &windows_app_data.join("LocalLow").to_string_lossy())
            .replace("{SteamUserData}", &linux_app_data.join("Steam/userdata/*").to_string_lossy())
            .replace("{XDGConfig}", &config().to_string_lossy())
            .replace("{XDGData}", &linux_app_data.to_string_lossy())
            .into()
    } else {
        let app_data = config();
        let home_dir = home();

        let steam_directory = match steamlocate::SteamDir::locate() {
            Ok(steam_dir) => steam_dir.path().join("userdata/*"),
            Err(_) => PathBuf::from("C:/Program Files (x86)/Steam/userdata/*")
        };

        path
            .replace("{AppData}", &app_data.to_string_lossy())
            .replace("{Documents}", &home_dir.join("Documents").to_string_lossy())
            .replace("{Home}", &home_dir.to_string_lossy())
            .replace("{LocalAppData}", &app_data.join("Local").to_string_lossy())
            .replace("{LocalLow}", &app_data.join("LocalLow").to_string_lossy())
            .replace("{SteamUserData}", &*steam_directory.to_string_lossy())
            .into()
    }
}

pub fn shrink_path(path: &str, prefix: Option<&PathBuf>) -> PathBuf {
    if cfg!(unix) {
        let wine_prefix = prefix.unwrap();
        let drive_c = wine_prefix.join("drive_c");
        let user = drive_c.join("users").join(std::env::var_os("USER").unwrap());
        let windows_app_data = user.join("AppData");
        let linux_app_data = app_data();

        path
            .replace(&*windows_app_data.join("LocalLow").to_string_lossy(), "{LocalLow}")
            .replace(&*windows_app_data.join("Local").to_string_lossy(), "{LocalAppData}")
            .replace(&*windows_app_data.to_string_lossy(), "{AppData}")
            .replace(&*user.join("Documents").to_string_lossy(), "{Documents}")
            .replace(&*user.to_string_lossy(), "{Home}")
            .replace(&*linux_app_data.join("Steam/userdata/*").to_string_lossy(), "{SteamUserData}")
            .replace(&*config().to_string_lossy(), "{XDGConfig}")
            .replace(&*linux_app_data.to_string_lossy(), "{XDGData}")
            .into()
    } else {
        let app_data = config();
        let home_dir = home();

        let steam_directory = match steamlocate::SteamDir::locate() {
            Ok(steam_dir) => steam_dir.path().join("userdata/*"),
            Err(_) => PathBuf::from("C:/Program Files (x86)/Steam/userdata/*")
        };

        path
            .replace(&*app_data.join("LocalLow").to_string_lossy(), "{LocalLow}")
            .replace(&*app_data.join("Local").to_string_lossy(), "{LocalAppData}")
            .replace(&*app_data.to_string_lossy(), "{AppData}")
            .replace(&*home_dir.join("Documents").to_string_lossy(), "{Documents}")
            .replace(&*home_dir.to_string_lossy(), "{Home}")
            .replace(&*steam_directory.to_string_lossy(), "{SteamUserData}")
            .into()
    }
}

pub fn get_size(path: &PathBuf) -> u64 {
    let mut size = 0;

    for entry in read_dir(path).unwrap() {
        let dir_entry = entry.unwrap();
        let entry_path = dir_entry.path();

        if entry_path.is_file() {
            size += entry_path.metadata().unwrap().len();
        } else if entry_path.is_dir() {
            size += get_size(&entry_path);
        }
    }

    size
}
