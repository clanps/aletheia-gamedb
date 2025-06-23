// SPDX-FileCopyrightText: 2025 Spencer
// SPDX-License-Identifier: AGPL-3.0-only

#[derive(Clone, serde::Deserialize)]
pub struct ForgejoRelease {
    pub body: String,
    pub tag_name: String,
    pub url: String
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

pub enum UpdateStatus {
    UpToDate,
    Available(ForgejoRelease),
}

pub fn check() -> Result<UpdateStatus> {
    let client = reqwest::blocking::Client::new();
    let response = client.get("https://git.usesarchbtw.lol/api/v1/repos/Spencer/aletheia/releases")
        .header(reqwest::header::USER_AGENT, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .send()?
        .error_for_status()?;

    let releases: Vec<ForgejoRelease> = response.json()?;

    let Some(latest_release) = releases.first() else {
        return Ok(UpdateStatus::UpToDate);
    };

    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let new_version = semver::Version::parse(&latest_release.tag_name).unwrap();

    if current_version < new_version {
        Ok(UpdateStatus::Available(latest_release.clone()))
    } else {
        Ok(UpdateStatus::UpToDate)
    }
}
