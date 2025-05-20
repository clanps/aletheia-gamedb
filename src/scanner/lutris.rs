// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::dirs::{app_data, home};
use super::{Game, Scanner};

pub struct LutrisScanner;

impl Scanner for LutrisScanner {
    fn get_games() -> Vec<Game> {
        let mut games = vec![];
        let lutris_db_path = app_data().join("lutris/pga.db");
        let lutris_db_path_flatpak = home().join(".var/app/net.lutris.Lutris/data/lutris/pga.db");

        let con = if lutris_db_path.exists() {
            rusqlite::Connection::open(lutris_db_path).unwrap()
        } else if lutris_db_path_flatpak.exists() {
            rusqlite::Connection::open(lutris_db_path_flatpak).unwrap()
        } else {
            return games;
        };

        let mut stmt = con.prepare("SELECT name, directory FROM games").unwrap();
        let rows = stmt
            .query_map([], |row| {
                let name: String = row.get(0)?;
                let path: String = row.get(1)?;
                Ok((name, std::path::PathBuf::from(path)))
            })
            .unwrap();

        for row in rows {
            let (name, dir) = row.unwrap();
            games.push(Game { name, installation_dir: None, prefix: Some(dir), source: "Lutris".into() });
        }

        games
    }
}
