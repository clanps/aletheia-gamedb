#[derive(serde::Deserialize)]
struct ForgejoRelease {
    tag_name: String
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error)
}

pub type Result<T> = core::result::Result<T, Error>;

pub fn check() -> Result<bool> {
    let client = reqwest::blocking::Client::new();
    let response = client.get("https://git.usesarchbtw.lol/api/v1/repos/Spencer/aletheia/releases")
        .header(reqwest::header::USER_AGENT, concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .send()?
        .error_for_status()?;

    let releases: Vec<ForgejoRelease> = response.json()?;

    let Some(latest_release) = releases.first() else {
        return Ok(false);
    };

    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION")).unwrap();
    let new_version = semver::Version::parse(&latest_release.tag_name).unwrap();

    Ok(current_version < new_version)
}
