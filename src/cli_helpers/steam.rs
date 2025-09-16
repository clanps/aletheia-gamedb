// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

use crate::config::Config;
use crate::scanner::SteamScanner;
use std::io::{self, Write};

pub fn ensure_steam_account_selected(config: &Config) {
    let users = SteamScanner::get_users().unwrap();
    let user_count = users.len();

    if user_count == 1 {
        let (steam_id, user) = users.iter().next().unwrap();
        println!("Steam account ID not set, setting as {} ({steam_id})", user.persona_name);

        Config::save(&Config {
            custom_databases: config.custom_databases.clone(),
            save_dir: config.save_dir.clone(),
            steam_account_id: Some(SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string()),
            #[cfg(feature = "updater")]
            check_for_updates: config.check_for_updates
        });
    } else {
        println!("Multiple Steam accounts found. Please choose one:");

        for (i, (steam_id, user)) in users.iter().enumerate() {
            println!("{}. {} ({})", i + 1, user.persona_name, steam_id);
        }

        loop {
            print!("Enter your choice (1-{user_count}): ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Error reading input. Please try again.");
                continue;
            }

            let choice = input.trim().parse::<usize>();
            match choice {
                Ok(num) if num >= 1 && num <= user_count => {
                    let (steam_id, user) = users.iter().nth(num - 1).unwrap();
                    println!("Selected {} ({steam_id})", user.persona_name);

                    Config::save(&Config {
                        custom_databases: config.custom_databases.clone(),
                        save_dir: config.save_dir.clone(),
                        steam_account_id: Some(SteamScanner::id64_to_id3(steam_id.parse::<u64>().unwrap()).to_string()),
                        #[cfg(feature = "updater")]
                        check_for_updates: config.check_for_updates
                    });
                    break;
                }
                _ => eprintln!("Invalid choice. Please enter a number between 1 and {user_count}.")
            }
        }
    }
}
