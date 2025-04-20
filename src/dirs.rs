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
