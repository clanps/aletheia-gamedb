// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

fn main() {
    let config = slint_build::CompilerConfiguration::new()
        .with_style("material-dark".into());

    slint_build::compile_with_config("ui/app.slint", config).expect("Slint build failed.");

    #[cfg(all(windows, not(debug_assertions)))]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("resources/logo/windows.ico");
        res.compile().unwrap();
    }
}
