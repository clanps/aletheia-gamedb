// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::commands::Command;
use crate::config::Config;
use crate::gamedb;
use slint::{Model, ModelRc, VecModel};

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{}B", size)
    } else if size < 1024 * 1024 {
        format!("{:.1}KB", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1}MB", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub fn run(config: Config) {
    let app = App::new().unwrap();
    let app_weak = app.as_weak();
    let cfg = config.clone();
    let save_dir = config.save_dir.clone();

    app.global::<GameLogic>().on_refresh_games({
        let app = app_weak.upgrade().unwrap();

        move || {
            let mut games = gamedb::get_installed_games();
            games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            let ui_games: Vec<UiGame> = games.into_iter().map(|g| {
                let name = g.name;
                let backup_path = save_dir.join(&name);

                UiGame {
                    name: name.into(),
                    backup_size: if backup_path.exists() {
                        format_size(crate::dirs::get_size(&backup_path)).into()
                    } else {
                        "0MB".into()
                    },
                    source: g.source.into(),
                    selected: true
                }
            }).collect();

            let games_model = ModelRc::new(std::rc::Rc::new(VecModel::from(ui_games)));
            app.global::<GameLogic>().set_games(games_model.clone());
            app.global::<BackupScreenLogic>().set_filtered_games(games_model);
        }
    });

    app.global::<BackupScreenLogic>().on_filter({
        let app_weak = app.as_weak();

        move |query| {
            let app = app_weak.upgrade().unwrap();
            let games = app.global::<GameLogic>().get_games();

            if query.is_empty() {
                app.global::<BackupScreenLogic>().set_filtered_games(games);
                return;
            }

            let filtered_games: Vec<UiGame> = games.iter()
                .filter(|g| g.name.to_lowercase().contains(&query.to_lowercase()))
                .collect();

            app.global::<BackupScreenLogic>().set_filtered_games(ModelRc::new(std::rc::Rc::new(VecModel::from(filtered_games))));
        }
    });

    app.global::<BackupScreenLogic>().on_select_game({
        let app_weak = app.as_weak();

        move |game| {
            let app = app_weak.upgrade().unwrap();
            let selected_games_model = app.global::<BackupScreenLogic>().get_selected_games();
            let mut selected_games: Vec<UiGame> = selected_games_model.iter().collect();

            if let Some(index) = selected_games.iter().position(|g| g.name == game.name) {
                selected_games.remove(index);
            } else {
                selected_games.push(game);
            }

            app.global::<BackupScreenLogic>().set_selected_games(ModelRc::new(std::rc::Rc::new(VecModel::from(selected_games))));
        }
    });

    app.global::<BackupScreenLogic>().on_backup({
        let app = app_weak.upgrade().unwrap();

        move || {
            let selected_games_model = app.global::<BackupScreenLogic>().get_selected_games();
            let selected_games: Vec<UiGame> = selected_games_model.iter().collect();
            let selected_game_names: Vec<String> = selected_games
                .iter()
                .map(|game| game.name.to_string())
                .collect();

            crate::commands::backup::Backup::run(selected_game_names, &cfg);
        }
    });

    app.global::<GameLogic>().invoke_refresh_games();
    app.run().unwrap();
}
