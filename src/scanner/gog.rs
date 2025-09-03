// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use std::fs::File;
use std::path::{Path, PathBuf};
use super::{Game, Scanner};

pub struct GOGScanner;

#[derive(serde::Deserialize)]
struct GOGameInfo {
    name: String
}

impl Scanner for GOGScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];

        #[cfg(windows)]
        let gog_db_path = Path::new("C:/ProgramData/GOG.com/Galaxy/storage/galaxy-2.0.db");

        #[cfg(target_os = "macos")]
        let gog_db_path = Path::new("/Users/Shared/GOG.com/Galaxy/Storage/galaxy-2.0.db");

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
                log::error!("{} is missing a GOG info file.", dir.display());
                continue;
            }

            let Ok(game_info) = serde_json::from_reader::<File, GOGameInfo>(File::open(info_path).unwrap()) else {
                log::error!("Malformed GOG manifest in {}.", dir.display());
                continue;
            };

            #[cfg(windows)]
            games.push(Game { name: game_info.name, installation_dir: Some(dir), source: "GOG".to_owned() });

            #[cfg(target_os = "macos")]
            games.push(Game { name: game_info.name, installation_dir: Some(dir), prefix: None, source: "GOG".to_owned() });
        }

        games
    }
}

