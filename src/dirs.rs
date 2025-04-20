use std::path::PathBuf;

pub fn config() -> std::path::PathBuf {
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

pub fn expand_path(path: &str, prefix: Option<&PathBuf>) -> PathBuf {
    if cfg!(unix) {
        let wine_prefix = prefix.unwrap();
        let drive_c = wine_prefix.join("drive_c");
        let users = drive_c.join("users").join("*");
        let app_data = users.join("AppData");
        let documents = users.join("Documents");

        path
            .replace("{AppData}", &app_data.display().to_string())
            .replace("{Documents}", &documents.display().to_string())
            .replace("{Home}", &users.display().to_string())
            .replace("{LocalAppData}", &app_data.join("Local").display().to_string())
            .replace("{LocalLow}", &app_data.join("LocalLow").display().to_string())
            .replace("{SteamUserData}", "{SteamUserData}") // TODO
            .replace("{XDGConfig}", &config().display().to_string())
            .into()
    } else {
        todo!("Windows path expansion")
    }
}
