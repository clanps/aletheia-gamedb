// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::commands::{Args, Command};
use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use slint::{Model, ModelRc, VecModel};

#[allow(clippy::cast_precision_loss, reason = "Only used for UI")]
fn format_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1048576;
    const GB: u64 = 1073741824;

    if size < KB {
        format!("{size}B")
    } else if size < MB {
        format!("{:.1}KB", size as f64 / KB as f64)
    } else if size < GB {
        format!("{:.1}MB", size as f64 / MB as f64)
    } else {
        format!("{:.2}GB", size as f64 / GB as f64)
    }
}

pub fn run(config: &AletheiaConfig) {
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
                let backup_path = save_dir.join(name.replace(':', ""));

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
            // In a perfect world, Slint would have a way to filter in their markdown language so I could avoid this
            app.global::<GameLogic>().set_games(games_model.clone());
            app.global::<GamesScreenLogic>().set_filtered_games(games_model.clone());
            app.global::<GamesScreenLogic>().set_selected_games(games_model);
        }
    });

    app.global::<GamesScreenLogic>().on_filter({
        let app_weak = app.as_weak();

        move |query| {
            let app = app_weak.upgrade().unwrap();
            let games = app.global::<GameLogic>().get_games();

            if query.is_empty() {
                app.global::<GamesScreenLogic>().set_filtered_games(games);
                return;
            }

            let filtered_games: Vec<UiGame> = games.iter()
                .filter(|g| g.name.to_lowercase().contains(&query.to_lowercase()))
                .collect();

            app.global::<GamesScreenLogic>().set_filtered_games(ModelRc::new(std::rc::Rc::new(VecModel::from(filtered_games))));
        }
    });

    app.global::<GamesScreenLogic>().on_select_all({
        let app_weak = app.as_weak();

        move |enabled| {
            let app = app_weak.upgrade().unwrap();
            let filtered_games_model = app.global::<GamesScreenLogic>().get_filtered_games();
            let updated_games: Vec<UiGame> = filtered_games_model.iter().map(|mut g| {
                g.selected = enabled;
                g
            }).collect();

            let updated_model = ModelRc::new(std::rc::Rc::new(VecModel::from(updated_games.clone())));
            app.global::<GamesScreenLogic>().set_filtered_games(updated_model.clone());
            app.global::<GamesScreenLogic>().set_selected_games(
                if enabled {
                    ModelRc::new(std::rc::Rc::new(VecModel::from(updated_games)))
                } else {
                    ModelRc::new(std::rc::Rc::new(VecModel::from(vec![])))
                }
            );
        }
    });


    app.global::<GamesScreenLogic>().on_select_game({
        let app_weak = app.as_weak();

        move |game| {
            let app = app_weak.upgrade().unwrap();
            let selected_games_model = app.global::<GamesScreenLogic>().get_selected_games();
            let mut selected_games: Vec<UiGame> = selected_games_model.iter().collect();

            if let Some(index) = selected_games.iter().position(|g| g.name == game.name) {
                selected_games.remove(index);
            } else {
                selected_games.push(game);
            }

            app.global::<GamesScreenLogic>().set_selected_games(ModelRc::new(std::rc::Rc::new(VecModel::from(selected_games))));
        }
    });

    app.global::<GamesScreenLogic>().on_perform_operation({
        let app = app_weak.upgrade().unwrap();
        let cfg = cfg.clone();

        move |action| {
            let selected_games_model = app.global::<GamesScreenLogic>().get_selected_games();
            let selected_games: Vec<UiGame> = selected_games_model.iter().collect();
            let selected_game_names = Args::parse(&selected_games
                .iter()
                .map(|game| game.name.to_string())
                .collect::<Vec<String>>()
            );

            if action == "backup" {
                crate::commands::Backup::run(selected_game_names, &cfg);
                app.global::<GameLogic>().invoke_refresh_games();
            } else {
                crate::commands::Restore::run(selected_game_names, &cfg);
            }
        }
    });

    app.global::<SettingsScreenLogic>().on_browse({
        let app_weak = app.as_weak();

        move || {
            let app = app_weak.upgrade().unwrap();

            slint::spawn_local(async move {
                if let Some(folder) = rfd::AsyncFileDialog::new()
                    .set_directory(crate::dirs::home())
                    .pick_folder()
                    .await
                {
                    let path = folder.path().to_string_lossy().to_string();
                    let mut cfg = app.global::<SettingsScreenLogic>().get_config();

                    cfg.save_dir = path.into();

                    app.global::<SettingsScreenLogic>().set_config(cfg);
                }
            }).unwrap();
        }
    });

    app.global::<SettingsScreenLogic>().on_save_config({
        move |cfg| {
            AletheiaConfig::save(AletheiaConfig { save_dir: (&cfg.save_dir).into() })
        }
    });

    app.global::<GameLogic>().invoke_refresh_games();
    app.global::<SettingsScreenLogic>().set_config(Config { save_dir: config.save_dir.to_string_lossy().to_string().into() });
    app.run().unwrap();
}
