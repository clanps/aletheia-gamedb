// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::read_to_string;
use std::path::PathBuf;
use super::{Game, Scanner};

pub struct GOGScanner;

#[derive(serde::Deserialize)]
struct GOGameInfo {
    name: String
}

impl Scanner for GOGScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];
        let gog_db_path = PathBuf::from("C:/ProgramData/GOG.com/Galaxy/storage/galaxy-2.0.db");

        if !gog_db_path.exists() {
            return games;
        }

        let con = rusqlite::Connection::open(gog_db_path).unwrap();
        let mut stmt = con.prepare("SELECT productId, installationPath FROM InstalledBaseProducts").unwrap();

        let rows = stmt
            .query_map([], |row| {
                let product_id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                Ok((product_id, PathBuf::from(path)))
            })
            .unwrap();

        for row in rows {
            let (product_id, dir) = row.unwrap();
            let info_path = dir.join(format!("goggame-{product_id}.info"));

            if !info_path.exists() {
                println!("{} is missing a GOG info file.", dir.display());
                continue;
            }

            let content = read_to_string(&info_path).unwrap_or_else(|_| panic!("Failed to read GOG manifest in {}.", dir.display()));
            let game_info: GOGameInfo = serde_json::from_str(&content).unwrap_or_else(|_| panic!("Malformed GOG manifest in {}.", dir.display()));

            games.push(Game { name: game_info.name, installation_dir: Some(dir), prefix: None, source: "GOG".to_owned() });
        }

        games
    }
}

