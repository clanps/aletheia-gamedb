use std::path::PathBuf;

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
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap();

        path
            .replace("{AppData}", &app_data.to_string_lossy())
            .replace("{Documents}", &home.join("Documents").to_string_lossy())
            .replace("{Home}", &home.to_string_lossy())
            .replace("{LocalAppData}", &app_data.join("Local").to_string_lossy())
            .replace("{LocalLow}", &app_data.join("LocalLow").to_string_lossy())
            .replace("{SteamUserData}", "C:/Program Files (x86)/Steam/userdata/*")
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
            .replace(&*windows_app_data.join("Local").to_string_lossy(), "{LocalAppData}")
            .replace(&*windows_app_data.join("LocalLow").to_string_lossy(), "{LocalLow}")
            .replace(&*windows_app_data.to_string_lossy(), "{AppData}")
            .replace(&*user.join("Documents").to_string_lossy(), "{Documents}")
            .replace(&*user.to_string_lossy(), "{Home}")
            .replace(&*linux_app_data.join("Steam/userdata/*").to_string_lossy(), "{SteamUserData}")
            .replace(&*config().to_string_lossy(), "{XDGConfig}")
            .replace(&*linux_app_data.to_string_lossy(), "{XDGData}")
            .into()
    } else {
        let app_data = config();
        let home = std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap();

        path
            .replace(&*app_data.join("Local").to_string_lossy(), "{LocalAppData}")
            .replace(&*app_data.join("LocalLow").to_string_lossy(), "{LocalLow}")
            .replace(&*app_data.to_string_lossy(), "{AppData}")
            .replace(&*home.join("Documents").to_string_lossy(), "{Documents}")
            .replace(&*home.to_string_lossy(), "{Home}")
            .replace("C:/Program Files (x86)/Steam/userdata/*", "{SteamUserData}")
            .into()
    }
}
