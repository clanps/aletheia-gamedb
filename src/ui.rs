// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::config::Config as AletheiaConfig;
use crate::gamedb;
use crate::operations::{backup_game, restore_game};
use crate::scanner::SteamScanner;
use slint::{Model, ModelRc, SharedString, VecModel};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::rc::Rc;

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
    #[cfg(all(feature = "updater", not(debug_assertions)))]
    if config.check_for_updates {
        if let Ok(updater::UpdateStatus::Available(release)) = updater::check() {
            let updater_window = Updater::new().unwrap();
            let updater_logic = updater_window.global::<UpdaterLogic>();

            slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

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

    let app = App::new().unwrap();
    let cfg = Rc::new(RefCell::new(config.clone()));
    let save_dir = config.borrow().save_dir.clone();

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
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let selected_names: HashSet<String> = games_screen_logic
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

            let ui_games_model = ModelRc::new(VecModel::from(ui_games));
            let ui_selected_games: Vec<UiGame> = ui_games_model.iter()
                .filter_map(|game| game.selected.then(|| game.clone()))
                .collect();

            // In a perfect world, Slint would have a way to filter in their markdown language so I could avoid this
            app_weak.global::<GameLogic>().set_games(ui_games_model.clone());
            games_screen_logic.set_filtered_games(ui_games_model.clone());
            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(ui_selected_games)));
            games_screen_logic.set_all_filtered_selected(ui_games_model.row_count() > 0 && ui_games_model.iter().all(|game| game.selected));
        }
    });

    games_screen_logic.on_filter({
        let app_weak = app.as_weak().unwrap();

        move |query| {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let games = app_weak.global::<GameLogic>().get_games();
            let selected_names: HashSet<String> = games_screen_logic
                .get_selected_games().iter().map(|g| g.name.to_string()).collect();

            let filtered_games: Vec<UiGame> = games.iter()
                .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query.to_lowercase()))
                .map(|mut g| {
                    g.selected = selected_names.contains(&g.name.to_string());
                    g
                })
                .collect();

            let all_filtered_selected = !filtered_games.is_empty() && filtered_games.iter().all(|g| g.selected);

            games_screen_logic.set_filtered_games(ModelRc::new(VecModel::from(filtered_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_select_all({
        let app_weak = app.as_weak().unwrap();

        move |enabled| {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let filtered_games_model = games_screen_logic.get_filtered_games();
            let updated_games: Vec<UiGame> = filtered_games_model.iter().map(|mut g| {
                g.selected = enabled;
                g
            }).collect();
            let updated_games_names: HashSet<&SharedString> = updated_games.iter().map(|g| &g.name).collect();
            let mut selected_games: Vec<UiGame> = games_screen_logic.get_selected_games().iter().collect();

            selected_games.retain(|g| !updated_games_names.contains(&g.name));
            if enabled {
                selected_games.extend(updated_games.iter().cloned());
            }

            let all_filtered_selected = enabled && !updated_games.is_empty();
            games_screen_logic.set_filtered_games(ModelRc::new(VecModel::from(updated_games)));
            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(selected_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_select_game({
        let app_weak = app.as_weak().unwrap();

        move |game| {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let selected_games_model = games_screen_logic.get_selected_games();
            let mut selected_games: Vec<UiGame> = selected_games_model.iter().collect();

            if let Some(index) = selected_games.iter().position(|g| g.name == game.name) {
                selected_games.remove(index);
            } else {
                selected_games.push(game);
            }

            let selected_games_names: HashSet<String> = selected_games.iter().map(|g| g.name.to_string()).collect();

            let filtered_games_model = games_screen_logic.get_filtered_games();
            let all_filtered_selected = filtered_games_model.row_count() > 0 &&
                filtered_games_model.iter().all(|g| selected_games_names.contains(&g.name.to_string()));

            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(selected_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_perform_operation({
        let app_weak = app.as_weak().unwrap();
        let cfg = Rc::clone(&cfg);

        move |action| {
            let cfg = cfg.as_ref().borrow();
            let notification_logic = app_weak.global::<NotificationLogic>();
            let selected_games_model = app_weak.global::<GamesScreenLogic>().get_selected_games();
            let selected_games: Vec<UiGame> = selected_games_model.iter().collect();
            let installed_games = gamedb::get_installed_games();

            if cfg.steam_account_id.is_none() && selected_games.iter().any(|g| g.source == "Steam") {
                notification_logic.invoke_show_warning("Set your Steam account in Settings.".into());
                return;
            }

            if action == "backup" {
                let game_db = gamedb::parse();

                for ui_game in &selected_games {
                    let game = installed_games.iter().find(|g| *g.name == *ui_game.name).unwrap();
                    if let Err(e) = backup_game(game, &cfg, &game_db[&game.name]) {
                        log::error!("Failed to backup {}.\n{e}", &game.name);
                    } else {
                        log::info!("Successfully backed up {}.", &game.name);
                    }
                }

                app_weak.global::<GameLogic>().invoke_refresh_games();
                notification_logic.invoke_show_success(format!("Backed up {} games", selected_games.len()).into());
            } else {
                if !cfg.save_dir.exists() {
                    notification_logic.invoke_show_error("Backup directory does not exist.".into());
                    return;
                }

                for ui_game in &selected_games {
                    let game_name = ui_game.name.to_string();
                    let game_dir = cfg.save_dir.join(game_name.replace(':', ""));

                    if !game_dir.exists() || !game_dir.is_dir() {
                        log::error!("Backup directory for {game_name} doesn't exist.");
                        continue;
                    }

                    let manifest_path = game_dir.join("aletheia_manifest.yaml");

                    if !manifest_path.exists() {
                        log::error!("{game_name} is missing a manifest file.");
                        continue;
                    }

                    let manifest_content = read_to_string(manifest_path).unwrap();
                    let Ok(manifest) = serde_yaml::from_str::<gamedb::GameInfo>(&manifest_content) else {
                        log::error!("Failed to parse {game_name}'s manifest.");
                        continue;
                    };

                    if let Err(e) = restore_game(&game_dir, &manifest, &installed_games, &cfg) {
                        log::error!("Failed to restore {}: {e}", manifest.name);
                    } else {
                        log::info!("Successfully restored {game_name}.");
                    }
                }

                notification_logic.invoke_show_success(format!("Restored {} games", selected_games.len()).into());
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
        let cfg = Rc::clone(&cfg);

        move |ui_cfg| {
            let notification_logic = app_weak.global::<NotificationLogic>();
            let current_config = cfg.as_ref().borrow().clone();
            let new_config = AletheiaConfig {
                custom_databases: ui_cfg.custom_databases.iter().map(Into::into).collect(),
                save_dir: (&ui_cfg.save_dir).into(),
                steam_account_id: (!ui_cfg.steam_account_id.is_empty()).then(|| (&ui_cfg.steam_account_id).into()),
                #[cfg(feature = "updater")]
                check_for_updates: ui_cfg.check_for_updates
            };

            if current_config == new_config {
                notification_logic.invoke_show_info("Settings are already up to date.".into());
            } else {
                AletheiaConfig::save(&new_config);
                *cfg.borrow_mut() = new_config;
                notification_logic.invoke_show_success("Successfully saved settings.".into());
            }
        }
    });

    settings_screen_logic.on_get_steam_users({
        let app_weak = app.as_weak().unwrap();

        move || {
            let settings_logic = app_weak.global::<SettingsScreenLogic>();

            if let Some(users) = SteamScanner::get_users() {
                let mut options: Vec<DropdownOption> = users.into_iter()
                    .map(|(steam_id, user)| DropdownOption {
                        label: user.persona_name.into(),
                        value: SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string().into()
                    })
                    .collect();

                if options.len() > 1 {
                    options.sort_by(|a, b| a.label.cmp(&b.label));
                }
                settings_logic.set_steam_account_options(ModelRc::new(VecModel::from(options)));
            }
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

    game_logic.invoke_refresh_games();
    app_logic.set_version(env!("CARGO_PKG_VERSION").into());

    #[cfg(feature = "updater")]
    settings_screen_logic.set_show_update_settings(true);

    settings_screen_logic.invoke_get_steam_users();

    let mut steam_account_id = match SteamScanner::get_users() {
        Some(users) if users.len() == 1 => users.keys().next().unwrap().clone(),
        Some(users) => {
            let cfg_id3 = config.steam_account_id.as_deref().unwrap_or_default();
            if cfg_id3.is_empty() {
                "".into()
            } else {
                users.keys()
                    .find(|id| SteamScanner::id64_to_id3(id.parse::<u64>().unwrap()).to_string() == cfg_id3)
                    .cloned()
                    .unwrap_or_default()
            }
        }
        None => "".into()
    };

    if !steam_account_id.is_empty() {
        println!("{}", steam_account_id);
        let id3 = SteamScanner::id64_to_id3(steam_account_id.parse::<u64>().unwrap()).to_string();
        steam_account_id = id3.clone();

        if Some(&id3) != config.steam_account_id.as_ref() {
            let new_config = AletheiaConfig {
                custom_databases: config.custom_databases.clone(),
                save_dir: config.save_dir.clone(),
                steam_account_id: Some(id3),
                #[cfg(feature = "updater")]
                check_for_updates: config.check_for_updates
            };
            AletheiaConfig::save(&new_config);
            *cfg.borrow_mut() = new_config;
        }
    }

    settings_screen_logic.set_config(Config {
        custom_databases: ModelRc::new(VecModel::from(config.custom_databases.iter().map(Into::into).collect::<Vec<_>>())),
        save_dir: config.save_dir.to_string_lossy().to_string().into(),
        steam_account_id: steam_account_id.into(),
        #[cfg(feature = "updater")]
        check_for_updates: config.check_for_updates,
        #[cfg(not(feature = "updater"))]
        check_for_updates: false
    });

    app.run().unwrap();
}
