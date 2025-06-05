# Aletheia
## Game Save Backup and Restore Utility

#### Supported Launchers
-   Heroic Games - GOG (Linux & Windows)
-   Lutris (Linux)
-   Steam (Linux & Windows)
-   GOG Galaxy (Windows)

## Contributing
### GameDB
Add game save locations in `resources/gamedb.yaml`. Include Linux paths if the game supports Linux. Game titles are based on their Steam names. The following placeholders can be used:

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
