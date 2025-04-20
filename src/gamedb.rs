const GAMEDB_YAML: &str = include_str!("../resources/gamedb.yaml");

#[derive(Debug, serde::Deserialize)]
pub struct GameDbEntry {
    pub files: GameFiles
}

#[derive(Debug, serde::Deserialize)]
pub struct GameFiles {
    pub windows: Option<Vec<String>>,
    pub linux: Option<Vec<String>>
}

pub fn parse() -> std::collections::HashMap<String, GameDbEntry> {
    serde_yaml::from_str(GAMEDB_YAML).expect("Failed to parse GameDB.")
}
