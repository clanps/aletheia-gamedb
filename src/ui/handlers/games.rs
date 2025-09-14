// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config as AletheiaConfig;
use crate::ui::app::{App, GameLogic, GamesScreenLogic, NotificationLogic, UiGame};
use crate::gamedb;
use crate::operations::{backup_game, restore_game};
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use std::fs::File;
use std::rc::Rc;
use std::cell::RefCell;

#[expect(clippy::too_many_lines, reason = "This is as simple as it's going to get")]
pub fn setup(app: &slint::Weak<App>, config: &Rc<RefCell<AletheiaConfig>>) {
    let app = app.upgrade().unwrap();
    let game_logic = app.global::<GameLogic>();
    let games_screen_logic = app.global::<GamesScreenLogic>();
    let save_dir = config.borrow().save_dir.clone();

    game_logic.on_refresh_games({
        let app_weak = app.as_weak().unwrap();

        move || {
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let selected_games = games_screen_logic.get_selected_games();
            let select_all = selected_games.row_count() == 0;

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
                    selected: select_all || selected_games.iter().any(|selected| selected.name.as_str() == name)
                }
            }).collect();

            let ui_games_model = ModelRc::new(VecModel::from(ui_games));
            let ui_selected_games: Vec<UiGame> = ui_games_model.iter()
                .filter(|game| game.selected)
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
            let query_lower = query.to_lowercase();
            let games_screen_logic = app_weak.global::<GamesScreenLogic>();
            let games = app_weak.global::<GameLogic>().get_games();
            let selected_games = games_screen_logic.get_selected_games();
            let filtered_games: Vec<UiGame> = games.iter()
                .filter(|g| query.is_empty() || g.name.to_lowercase().contains(&query_lower))
                .map(|mut g| {
                    g.selected = selected_games.iter().any(|selected| selected.name == g.name);
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
            let mut selected_games: Vec<UiGame> = games_screen_logic.get_selected_games().iter().collect();

            selected_games.retain(|g| !updated_games.iter().any(|updated| updated.name == g.name));
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

            let filtered_games_model = games_screen_logic.get_filtered_games();
            let all_filtered_selected = filtered_games_model.row_count() > 0 &&
                filtered_games_model.iter().all(|g| selected_games.iter().any(|selected| selected.name == g.name));

            games_screen_logic.set_selected_games(ModelRc::new(VecModel::from(selected_games)));
            games_screen_logic.set_all_filtered_selected(all_filtered_selected);
        }
    });

    games_screen_logic.on_perform_operation({
        let app_weak = app.as_weak().unwrap();
        let cfg = Rc::clone(config);

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
                    let game_name = &ui_game.name;
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

                    let Ok(manifest) = serde_yaml::from_reader::<File, gamedb::GameInfo>(File::open(manifest_path).unwrap()) else {
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

    game_logic.invoke_refresh_games();
}

#[expect(clippy::cast_precision_loss, reason = "Only used for UI")]
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
