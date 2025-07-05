// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::env::{home_dir, var_os};
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

pub fn cache() -> PathBuf {
    if cfg!(unix) {
        var_os("XDG_CACHE_HOME")
            .map_or_else(|| home().join(".cache"), PathBuf::from)
            .join("aletheia")
    } else {
        app_data().join("aletheia/cache")
    }
}

pub fn config() -> PathBuf {
    if cfg!(unix) {
        var_os("XDG_CONFIG_HOME")
            .map_or_else(|| home().join(".config"), PathBuf::from)
    } else {
        var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap()
    }
}

pub fn app_data() -> PathBuf {
    if cfg!(unix) {
        var_os("XDG_DATA_HOME")
            .map_or_else(|| home().join(".local/share"), PathBuf::from)
    } else {
        var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap()
    }
}

pub fn home() -> PathBuf {
    home_dir().unwrap()
}

fn expand_path_components(path: &Path, replacements: &[(&str, PathBuf)]) -> PathBuf {
    let mut result = PathBuf::new();

    for component in path.components() {
        let component_os = component.as_os_str();
        let mut replaced = false;

        for (pattern, replacement) in replacements {
            if component_os == *pattern {
                result.push(replacement);
                replaced = true;
                break;
            }
        }

        if !replaced {
            result.push(component);
        }
    }

    result
}

fn shrink_path_components(path: &Path, replacements: &[(&str, PathBuf)]) -> PathBuf {
    for (pattern, replacement) in replacements {
        if let Ok(stripped) = path.strip_prefix(replacement) {
            let mut new_path = PathBuf::from(pattern);
            new_path.push(stripped);
            return new_path;
        }
    }

    path.to_path_buf()
}

fn path_contains_subpath(haystack: &Path, needle: &str) -> bool {
    haystack
        .ancestors()
        .any(|ancestor| ancestor.ends_with(needle))
}

pub fn expand_path(path: &Path, installation_dir: Option<&PathBuf>, prefix: Option<&PathBuf>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    if cfg!(unix) {
        let linux_app_data = app_data();

        if let Some(wine_prefix) = prefix {
            let username = if path_contains_subpath(wine_prefix, "Steam/steamapps/compatdata") {
                OsString::from("steamuser")
            } else {
                std::env::var_os("USER").unwrap()
            };

            let drive_c = wine_prefix.join("drive_c");
            let user = drive_c.join("users").join(username);
            let windows_app_data = user.join("AppData");
            let documents = user.join("Documents");

            replacements.extend([
                ("{AppData}", windows_app_data.join("Roaming")),
                ("{Documents}", documents),
                ("{Home}", user),
                ("{LocalAppData}", windows_app_data.join("Local")),
                ("{LocalLow}", windows_app_data.join("LocalLow")),
                ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
                ("{SteamUserData}", linux_app_data.join("Steam/userdata/[0-9]*"))
            ]);
        }

        replacements.extend([
            ("{XDGConfig}", config()),
            ("{XDGData}", linux_app_data)
        ]);
    } else {
        let roaming_app_data = config();
        let local_app_data = app_data();
        let home_dir = home();

        let steam_directory = match steamlocate::SteamDir::locate() {
            Ok(steam_dir) => steam_dir.path().join("userdata/[0-9]*"),
            Err(_) => PathBuf::from("C:/Program Files (x86)/Steam/userdata/[0-9]*")
        };

        replacements.extend([
            ("{AppData}", roaming_app_data),
            ("{Documents}", home_dir.join("Documents")),
            ("{Home}", home_dir),
            ("{LocalAppData}", local_app_data.clone()),
            ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
            ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
            ("{SteamUserData}", steam_directory)
        ]);
    }

    expand_path_components(path, &replacements)
}

pub fn shrink_path(path: &Path, installation_dir: Option<&PathBuf>, prefix: Option<&PathBuf>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    if cfg!(unix) {
        let linux_app_data = app_data();

        if let Some(wine_prefix) = prefix {
            let username = if path_contains_subpath(wine_prefix, "Steam/steamapps/compatdata") {
                OsString::from("steamuser")
            } else {
                var_os("USER").unwrap()
            };

            let drive_c = wine_prefix.join("drive_c");
            let user = drive_c.join("users").join(username);
            let windows_app_data = user.join("AppData");

            replacements.extend([
                ("{LocalLow}", windows_app_data.join("LocalLow")),
                ("{LocalAppData}", windows_app_data.join("Local")),
                ("{AppData}", windows_app_data.join("Roaming")),
                ("{Documents}", user.join("Documents")),
                ("{Home}", user),
                ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
                ("{SteamUserData}", linux_app_data.join("Steam/userdata/[0-9]*"))
            ]);
        }

        replacements.extend([
            ("{XDGConfig}", config()),
            ("{XDGData}", linux_app_data)
        ]);
    } else {
        let roaming_app_data = config();
        let local_app_data = config();
        let home_dir = home();

        let steam_directory = match steamlocate::SteamDir::locate() {
            Ok(steam_dir) => steam_dir.path().join("userdata/[0-9]*"),
            Err(_) => PathBuf::from("C:/Program Files (x86)/Steam/userdata/[0-9]*")
        };

        replacements.extend([
            ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
            ("{LocalAppData}", local_app_data.clone()),
            ("{AppData}", roaming_app_data),
            ("{Documents}", home_dir.join("Documents")),
            ("{Home}", home_dir),
            ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
            ("{SteamUserData}", steam_directory)
        ]);
    }

    shrink_path_components(path, &replacements)
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
