// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::gamedb;
use slint::{Model, ModelRc, VecModel};

pub fn run() {
    let app = App::new().unwrap();
    let app_weak = app.as_weak();

    app.global::<GameLogic>().on_refresh_games(move || {
        let app = app_weak.upgrade().unwrap();

        let mut games = gamedb::get_installed_games();
        games.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let ui_games: Vec<UiGame> = games.into_iter().map(|g| UiGame {
            name: g.name.into(),
            backup_size: "0MB".into(),
            source: g.source.into(),
            selected: true
        }).collect();

        let games_model = ModelRc::new(std::rc::Rc::new(VecModel::from(ui_games)));
        app.global::<GameLogic>().set_games(games_model.clone());
        app.global::<BackupScreenLogic>().set_filtered_games(games_model);
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

    app.global::<GameLogic>().invoke_refresh_games();
    app.run().unwrap();
}
