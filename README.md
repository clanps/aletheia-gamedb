<!--
SPDX-FileCopyrightText: 2025 Spencer
SPDX-License-Identifier: AGPL-3.0-only
-->

<p align="center">
  <br><img src="https://raw.githubusercontent.com/Spencer-0003/aletheia/refs/heads/master/resources/logo/512x512.png" width="220" /><br/>
  <b>Aletheia</b>
</p>

## What is Aletheia?
Aletheia is a cross-platform game save sync tool designed to help you easily back up and restore game saves across multiple game launchers and devices.

#### Supported Launchers
-   Heroic Games - GOG (Linux & Windows)
-   Lutris (Linux)
-   Steam (Linux & Windows)
-   GOG Galaxy (Windows)

## Contributing
### GameDB
Add game save locations in `resources/gamedb.yaml`. Include Linux paths if the game supports Linux. Game titles are based on their GOG names, but Steam and itch.io games are also accepted. The following placeholders can be used:

| Placeholder       | Description                                                                                   |
|-------------------|-----------------------------------------------------------------------------------------------|
| `{GameRoot}`      | Root directory of the game installation                                                       |
| `{AppData}`       | Roaming AppData folder on Windows                                                             |
| `{LocalAppData}`  | Local AppData folder on Windows                                                               |
| `{LocalLow}`      | LocalLow AppData folder on Windows                                                            |
| `{Documents}`     | User’s documents directory                                                                    |
| `{Home}`          | User’s home directory                                                                         |
| `{XDGConfig}`     | Linux XDG config directory                                                                    |
| `{XDGData}`       | Linux XDG data directory                                                                      |
| `{GOGAppData}`    | GOG application data directory                                                                |
| `{SteamUserData}` | Steam userdata directory (supports wildcard for multiple user profiles)                       |

## Support the Project
If you find Aletheia useful and would like to support its development, consider donating.

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/R6R41GPTPU)

- Bitcoin: `bc1q9gdrmsakekn86k2ejlhah9a68vjjzl0yyvxc97`
- Ethereum: `0x67e537EE6A2a865F22C0e7e036DEaD6f1e89e315`
- Litecoin: `ltc1q4wpjx0dacnqr24dnh4zjtw8mjguhavyqhuam3s`
- Monero: `45HTdxmYdmmH58mne2MjjK8nshYeFDh9JhXs8MvjG7bH6S8yHEUp9fhN1PFGzoyMmWWiKivMtqBx9BXbDvpxSfH3DM3v4jE`

## Credits
- `@unidentified:usesarchbtw.lol` -> Helped with UI design
- `@clanps:usesarchbtw.lol` -> Designed the logo
