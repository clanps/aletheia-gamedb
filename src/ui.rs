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

#[allow(clippy::too_many_lines, reason = "I will refactor this 'at some point'")]
pub fn run(config: &AletheiaConfig) {
    let app = App::new().unwrap();
    let cfg = config.clone();
    let save_dir = config.save_dir.clone();

    app.global::<AppLogic>().on_open_url(move |url| {
        #[cfg(unix)]
        std::process::Command::new("xdg-open").arg(url).spawn().ok();

        #[cfg(windows)]
        std::process::Command::new("cmd").args(["/c", "start", &url]).spawn().ok();
    });

    app.global::<GameLogic>().on_refresh_games({
        let app_weak = app.as_weak().unwrap();

        move || {
            let selected_names: std::collections::HashSet<String> = app_weak.global::<GamesScreenLogic>()
                .get_selected_games().iter().map(|g| g.name.to_string()).collect();
            let select_all = selected_names.is_empty();

            let mut games = gamedb::get_installed_games();
            games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

            let ui_games: Vec<UiGame> = games.into_iter().map(|g| {
                let name = g.name;
                let backup_path = save_dir.join(name.replace(':', ""));

                UiGame {
                    name: name.clone().into(),
                    backup_size: if backup_path.exists() {
                        format_size(crate::dirs::get_size(&backup_path)).into()
                    } else {
                        "0B".into()
                    },
                    source: g.source.into(),
                    selected: select_all || selected_names.contains(&name)
                }
            }).collect();

            let games_model = ModelRc::new(VecModel::from(ui_games));
            // In a perfect world, Slint would have a way to filter in their markdown language so I could avoid this
            app_weak.global::<GameLogic>().set_games(games_model.clone());
            app_weak.global::<GamesScreenLogic>().set_filtered_games(games_model.clone());
            app_weak.global::<GamesScreenLogic>().set_selected_games(ModelRc::new(VecModel::from(games_model.iter().filter(|g| g.selected).collect::<Vec<UiGame>>())));
        }
    });

    app.global::<GamesScreenLogic>().on_filter({
        let app_weak = app.as_weak().unwrap();

        move |query| {
            let games = app_weak.global::<GameLogic>().get_games();

            if query.is_empty() {
                app_weak.global::<GamesScreenLogic>().set_filtered_games(games);
                return;
            }

            let filtered_games: Vec<UiGame> = games.iter()
                .filter(|g| g.name.to_lowercase().contains(&query.to_lowercase()))
                .collect();

            app_weak.global::<GamesScreenLogic>().set_filtered_games(ModelRc::new(VecModel::from(filtered_games)));
        }
    });

    app.global::<GamesScreenLogic>().on_select_all({
        let app_weak = app.as_weak().unwrap();

        move |enabled| {
            let filtered_games_model = app_weak.global::<GamesScreenLogic>().get_filtered_games();
            let updated_games: Vec<UiGame> = filtered_games_model.iter().map(|mut g| {
                g.selected = enabled;
                g
            }).collect();

            let updated_model = ModelRc::new(VecModel::from(updated_games.clone()));
            app_weak.global::<GamesScreenLogic>().set_filtered_games(updated_model.clone());
            app_weak.global::<GamesScreenLogic>().set_selected_games(
                if enabled {
                    ModelRc::new(VecModel::from(updated_games))
                } else {
                    ModelRc::new(VecModel::from(vec![]))
                }
            );
        }
    });


    app.global::<GamesScreenLogic>().on_select_game({
        let app_weak = app.as_weak().unwrap();

        move |game| {
            let selected_games_model = app_weak.global::<GamesScreenLogic>().get_selected_games();
            let mut selected_games: Vec<UiGame> = selected_games_model.iter().collect();

            if let Some(index) = selected_games.iter().position(|g| g.name == game.name) {
                selected_games.remove(index);
            } else {
                selected_games.push(game);
            }

            app_weak.global::<GamesScreenLogic>().set_selected_games(ModelRc::new(VecModel::from(selected_games)));
        }
    });

    app.global::<GamesScreenLogic>().on_perform_operation({
        let app_weak = app.as_weak().unwrap();
        let cfg = cfg.clone();

        move |action| {
            let selected_games_model = app_weak.global::<GamesScreenLogic>().get_selected_games();
            let selected_games: Vec<UiGame> = selected_games_model.iter().collect();
            let selected_game_names = Args::parse(&selected_games
                .iter()
                .map(|game| game.name.to_string())
                .collect::<Vec<String>>()
            );

            if action == "backup" {
                crate::commands::Backup::run(selected_game_names, &cfg);
                app_weak.global::<GameLogic>().invoke_refresh_games();
                app_weak.global::<NotificationLogic>().invoke_show_success(format!("Backed up {} games", selected_games.len()).into());
            } else {
                crate::commands::Restore::run(selected_game_names, &cfg);
                app_weak.global::<NotificationLogic>().invoke_show_success(format!("Restored {} games", selected_games.len()).into());
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
            AletheiaConfig::save(&AletheiaConfig { save_dir: (&cfg.save_dir).into() });
        }
    });

    app.global::<GameLogic>().invoke_refresh_games();
    app.global::<AppLogic>().set_version(env!("CARGO_PKG_VERSION").into());
    app.global::<SettingsScreenLogic>().set_config(Config { save_dir: config.save_dir.to_string_lossy().to_string().into() });
    app.run().unwrap();
}
