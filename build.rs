// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

fn main() {
    let config = slint_build::CompilerConfiguration::new();

    slint_build::compile_with_config("ui/app.slint", config).expect("Slint build failed.");
}
