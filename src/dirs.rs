// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::env::{home_dir, var_os};
use std::ffi::OsString;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub fn cache() -> PathBuf {
    var_os("XDG_CACHE_HOME")
        .map_or_else(|| home().join("Library/caches"), PathBuf::from)
        .join("moe.spencer.aletheia")
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn cache() -> PathBuf {
    var_os("XDG_CACHE_HOME")
        .map_or_else(|| home().join(".cache"), PathBuf::from)
        .join("aletheia")
}

#[cfg(windows)]
pub fn cache() -> PathBuf {
    app_data().join("aletheia/cache")
}

#[cfg(target_os = "macos")]
pub fn config() -> PathBuf {
    var_os("XDG_CONFIG_HOME")
        .map_or_else(|| home().join("Library/Preferences"), PathBuf::from)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn config() -> PathBuf {
    var_os("XDG_CONFIG_HOME")
         .map_or_else(|| home().join(".config"), PathBuf::from)
}

#[cfg(windows)]
pub fn config() -> PathBuf {
    var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap()
}

#[cfg(target_os = "macos")]
pub fn app_data() -> PathBuf {
    var_os("XDG_DATA_HOME")
        .map_or_else(|| home().join("Library/Application Support"), PathBuf::from)
}

#[cfg(all(unix, not(target_os = "macos")))]
pub fn app_data() -> PathBuf {
    var_os("XDG_DATA_HOME")
        .map_or_else(|| home().join(".local/share"), PathBuf::from)
}

#[cfg(windows)]
pub fn app_data() -> PathBuf {
    var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap()
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

#[cfg(all(unix, not(target_os = "macos")))]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

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

        let steam_user_data = steam_account_id.map_or_else(|| linux_app_data.join("Steam/userdata/[0-9]*"), |id| linux_app_data.join("Steam/userdata").join(id));

        replacements.extend([
            ("{AppData}", windows_app_data.join("Roaming")),
            ("{Documents}", documents),
            ("{Home}", user),
            ("{LocalAppData}", windows_app_data.join("Local")),
            ("{LocalLow}", windows_app_data.join("LocalLow")),
            ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
            ("{SteamUserData}", steam_user_data)
        ]);
    }

    replacements.extend([
        ("{XDGConfig}", config()),
        ("{XDGData}", linux_app_data)
    ]);

    expand_path_components(path, &replacements)
}

#[cfg(windows)]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let roaming_app_data = config();
    let local_app_data = app_data();
    let home_dir = home();

    let steam_user_data = {
        let base_path = steamlocate::SteamDir::locate()
            .map_or_else(|_| PathBuf::from("C:/Program Files (x86)/Steam"), |dir| dir.path().to_path_buf());

        let userdata_path = base_path.join("userdata");
        steam_account_id.map_or_else(|| userdata_path.join("[0-9]*"), |id| userdata_path.join(id))
    };

    replacements.extend([
        ("{AppData}", roaming_app_data),
        ("{Documents}", home_dir.join("Documents")),
        ("{Home}", home_dir),
        ("{LocalAppData}", local_app_data.clone()),
        ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
        ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
        ("{SteamUserData}", steam_user_data)
    ]);

    expand_path_components(path, &replacements)
}

#[cfg(target_os = "macos")]
pub fn expand_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
    let mut replacements: Vec<(&str, PathBuf)> = vec![];

    if let Some(install_dir) = installation_dir {
        replacements.push(("{GameRoot}", install_dir.to_owned()));
    }

    let application_support = home_dir().join("Library/Application Support"); // app_data is not used here as most games don't use the XDG spec on MacOS

    if let Some(wine_prefix) = prefix {
        let username = std::env::var_os("USER").unwrap();

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
            // ("{SteamUserData}", steam_user_data) // TODO: Find out where steam user data is stored on MacOS
        ]);
    } else {
        replacements.extend([
            ("{AppData}", application_support.clone()),
            ("{GOGAppData}", application_support.join("GOG.com/Galaxy/Applications"))
        ]);
    }

    expand_path_components(path, &replacements)
}

pub fn shrink_path(path: &Path, installation_dir: Option<&Path>, prefix: Option<&Path>, steam_account_id: Option<&str>) -> PathBuf {
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

            let steam_user_data = steam_account_id.map_or_else(|| linux_app_data.join("Steam/userdata/[0-9]*"), |id| linux_app_data.join("Steam/userdata").join(id));

            replacements.extend([
                ("{LocalLow}", windows_app_data.join("LocalLow")),
                ("{LocalAppData}", windows_app_data.join("Local")),
                ("{AppData}", windows_app_data.join("Roaming")),
                ("{Documents}", user.join("Documents")),
                ("{Home}", user),
                ("{GOGAppData}", windows_app_data.join("Local").join("GOG.com/Galaxy/Applications")),
                ("{SteamUserData}", steam_user_data)
            ]);
        } else if cfg!(target_os = "macos") {
            replacements.push(("{GOGAppData}", linux_app_data.join("GOG.com/Galaxy/Applications")));
        }

        replacements.extend([
            ("{XDGConfig}", config()),
            ("{XDGData}", linux_app_data)
        ]);
    } else {
        let roaming_app_data = config();
        let local_app_data = config();
        let home_dir = home();

        let steam_user_data = {
            let base_path = steamlocate::SteamDir::locate()
                .map_or_else(|_| PathBuf::from("C:/Program Files (x86)/Steam"), |dir| dir.path().to_path_buf());

            let userdata_path = base_path.join("userdata");
            steam_account_id.map_or_else(|| userdata_path.join("[0-9]*"), |id| userdata_path.join(id))
        };

        replacements.extend([
            ("{LocalLow}", local_app_data.parent().unwrap().join("LocalLow")),
            ("{LocalAppData}", local_app_data.clone()),
            ("{AppData}", roaming_app_data),
            ("{Documents}", home_dir.join("Documents")),
            ("{Home}", home_dir),
            ("{GOGAppData}", local_app_data.join("GOG.com/Galaxy/Applications")),
            ("{SteamUserData}", steam_user_data)
        ]);
    }

    shrink_path_components(path, &replacements)
}

pub fn get_size(path: &Path) -> u64 {
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
