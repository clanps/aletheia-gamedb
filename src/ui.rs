// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::commands::{Args, Command};
use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use slint::{Model, ModelRc, VecModel};

#[cfg(all(feature = "updater", not(debug_assertions)))]
use crate::updater;

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

    let app_logic = app.global::<AppLogic>();
    let game_logic = app.global::<GameLogic>();
    let games_screen_logic = app.global::<GamesScreenLogic>();
    let settings_screen_logic = app.global::<SettingsScreenLogic>();

    slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

    app_logic.on_open_url(move |url| {
        #[cfg(unix)]
        std::process::Command::new("xdg-open").arg(url).spawn().ok();

        #[cfg(windows)]
        std::process::Command::new("cmd").args(["/c", "start", &url]).spawn().ok();
    });

    game_logic.on_refresh_games({
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

    games_screen_logic.on_filter({
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

    games_screen_logic.on_select_all({
        let app_weak = app.as_weak().unwrap();

        move |enabled| {
            let filtered_games_model = app_weak.global::<GamesScreenLogic>().get_filtered_games();
            let updated_games: Vec<UiGame> = filtered_games_model.iter().map(|mut g| {
                g.selected = enabled;
                g
            }).collect();

            let updated_model = ModelRc::new(VecModel::from(updated_games.clone()));
            app_weak.global::<GamesScreenLogic>().set_filtered_games(updated_model.clone());
            app_weak.global::<GamesScreenLogic>().set_selected_games(ModelRc::new(VecModel::from(
                if enabled {
                    updated_games
                } else {
                    vec![]
                }
            )));
        }
    });


    games_screen_logic.on_select_game({
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

    games_screen_logic.on_perform_operation({
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

    settings_screen_logic.on_browse({
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

    settings_screen_logic.on_save_config({
        let app_weak = app.as_weak().unwrap();

        move |cfg| {
            AletheiaConfig::save(&AletheiaConfig {
                custom_databases: cfg.custom_databases.iter().map(Into::into).collect(),
                save_dir: (&cfg.save_dir).into(),
                #[cfg(feature = "updater")]
                check_for_updates: cfg.check_for_updates
            });

            app_weak.global::<NotificationLogic>().invoke_show_success("Successfully saved settings.".into());
        }
    });

    settings_screen_logic.on_update_gamedb({
        let app_weak = app.as_weak().unwrap();

        move || {
            let notification_logic = app_weak.global::<NotificationLogic>();

            match gamedb::update() {
                Ok(true) => notification_logic.invoke_show_success("Successfully updated GameDB.".into()),
                Ok(false) => notification_logic.invoke_show_info("GameDB is already up to date.".into()),
                Err(e) => {
                    notification_logic.invoke_show_error("Failed to update GameDB.".into());
                    log::error!("Error updating GameDB: {e}");
                }
            }
        }
    });

    #[cfg(all(feature = "updater", not(debug_assertions)))]
    if config.check_for_updates {
        if let Ok(updater::UpdateStatus::Available(release)) = updater::check() {
            let updater_window = Updater::new().unwrap();
            let updater_logic = updater_window.global::<UpdaterLogic>();

            updater_logic.set_current_version(env!("CARGO_PKG_VERSION").into());
            updater_logic.set_new_version(release.tag_name.into());
            updater_logic.set_changelog(release.body.into());

            updater_logic.on_skip_update({
                let updater_window = updater_window.as_weak().unwrap();

                move || updater_window.window().hide().unwrap()
            });

            updater_logic.on_download_update({
                let updater_window = updater_window.as_weak().unwrap();

                move || {
                    #[cfg(unix)]
                    std::process::Command::new("xdg-open").arg(release.url.clone()).spawn().ok();

                    #[cfg(windows)]
                    std::process::Command::new("cmd").args(["/c", "start", &release.url.clone()]).spawn().ok();

                    updater_window.window().hide().unwrap();
                }
            });

            updater_window.run().unwrap();

            if updater_logic.get_downloading() {
                return;
            }
        }
    }

    game_logic.invoke_refresh_games();
    app_logic.set_version(env!("CARGO_PKG_VERSION").into());

    #[cfg(feature = "updater")]
    settings_screen_logic.set_show_update_settings(true);

    settings_screen_logic.set_config(Config {
        custom_databases: ModelRc::new(VecModel::from(config.custom_databases.iter().map(Into::into).collect::<Vec<_>>())),
        save_dir: config.save_dir.to_string_lossy().to_string().into(),
        #[cfg(feature = "updater")]
        check_for_updates: config.check_for_updates,
        #[cfg(not(feature = "updater"))]
        check_for_updates: false
    });

    app.run().unwrap();
}
