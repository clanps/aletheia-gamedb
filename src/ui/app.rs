// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

slint::include_modules!();

use crate::config::Config as AletheiaConfig;
use super::{games, settings};
use std::cell::RefCell;
use std::rc::Rc;

#[cfg(all(feature = "updater", not(debug_assertions)))]
use crate::updater;

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
                    std::process::Command::new("cmd").args(["/c", "start", &release.url.clone()]).creation_flags(0x08000000).spawn().ok();

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
    let app_weak = app.as_weak();
    let cfg = Rc::new(RefCell::new(config.clone()));

    slint::set_xdg_app_id("moe.spencer.Aletheia").unwrap();

    setup_app_handlers(&app);
    games::setup(&app_weak, &cfg);
    settings::setup(&app_weak, &cfg);

    app.run().unwrap();
}

fn setup_app_handlers(app: &App) {
    let app_logic = app.global::<AppLogic>();

    app_logic.set_version(env!("CARGO_PKG_VERSION").into());
    app_logic.on_open_url(move |url| {
        #[cfg(unix)]
        std::process::Command::new("xdg-open").arg(url).spawn().ok();

        #[cfg(windows)]
        std::process::Command::new("cmd").args(["/c", "start", &url]).spawn().ok();
    });
}
