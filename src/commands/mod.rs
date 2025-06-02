// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[cfg(feature = "updater")]
mod check_for_update;

mod backup;
mod restore;
mod update;
mod update_custom;

#[cfg(feature = "updater")]
pub use check_for_update::CheckForUpdate;

pub use backup::Backup;
pub use restore::Restore;
pub use update::Update;
pub use update_custom::UpdateCustom;

pub struct Args {
    pub positional: Vec<String>,
    pub flags: Vec<Flag>
}

pub struct Flag {
    pub name: String,
    pub value: Option<String>
}

impl Flag {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: None
        }
    }

    pub fn with_value(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: Some(value.to_owned())
        }
    }
}

impl Args {
    pub fn parse(args: &[String]) -> Self {
        let mut positional = Vec::new();
        let mut flags = Vec::new();

        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];

            if let Some(name) = arg.strip_prefix("--") {
                let has_value = i + 1 < args.len() && !args[i + 1].starts_with('-');

                if has_value {
                    flags.push(Flag::with_value(name, &args[i + 1]));
                    i += 2;
                } else {
                    flags.push(Flag::new(name));
                    i += 1;
                }
            } else {
                positional.push(arg.clone());
                i += 1;
            }
        }

        Self { positional, flags }
    }

    #[allow(unused, reason = "Will be used in the future")]
    pub fn has_flag(&self, name: &str) -> bool {
        self.flags.iter().any(|f| f.name == name)
    }

    pub fn get_flag(&self, name: &str) -> Option<&Flag> {
        self.flags.iter().find(|f| f.name == name)
    }

    pub fn get_flag_value(&self, name: &str) -> Option<&String> {
        self.get_flag(name).and_then(|f| f.value.as_ref())
    }

    #[allow(unused, reason = "Will be used in the future")]
    pub fn flags_map(&self) -> std::collections::HashMap<String, Option<String>> {
        self.flags.iter()
            .map(|f| (f.name.clone(), f.value.clone()))
            .collect()
    }
}

pub trait Command {
    fn run(args: Args, config: &crate::config::Config);
}
