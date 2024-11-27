# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.4] - 2024-11-27

### Changed

- Update changelog

- Media Proxy
- Allow media token to expire
- Use dptree based dispatching
- Import ruma from matrix_sdk
- Refactor Rust files
- Update
- Redo some code
- 0.3.4


### Fixed

- Fix release action

## [0.3.3] - 2024-11-05

### Added

- Add proper recovery support
- Add nixpkgs command
- Add cross signing management

### Changed

- Update changelog

- Enable recovery instead
- Bump dependencies
- Apply Clippy suggestions
- Update docs
- Update changelog

- Ping-admin command
- Bump dependencies
- Move to workspace
- Use cynic to build GraphQL query
- Nixpkgs PR tracking
- Update
- Update flake builds
- 0.3.3


### Fixed

- Fix actions
- Fix code for changed matrix-rust-sdk API
- Fix user_id command
- Fix sending done status

## [0.3.2] - 2024-09-20

### Changed

- Update actions
- Update warning in avatar_changes
- 0.3.2


## [0.3.1] - 2024-09-20

### Added

- Add www prefix to pixiv links
- Add a warning to avatar_changes
- Add interactive-login feature
- Add changelog
- Add changelog workflow

### Changed

- Merge pull request #1 from Guanran928/pixiv-url


- Update Dev Container

- Use crane instead
- Migrate deprecated extension
- Make pixiv command respect config
- BiliBili extractor
- Update flakes
- Use the same toolchain for two devshells
- Update Flakes Rust toolchain
- Set RUST_SRC_PATH in flakes
- Allow specifiy directories in env var
- Systemd service
- Bump dependencies
- 0.3.1


### Fixed

- Use /artworks/{id} instead of /i/{id}
- Properly restart sync on error
- Fix Actions
- Fix ping command

### Removed

- Remove quote
- Remove Dev Containers

## [0.3.0] - 2024-05-15

### Changed

- Refactor config

- Move specials to reading config file

- Update nahida functions

- Update configuration docs

- Update nahida functions

- V0.3.0


### Fixed

- Fix title extractor

- Fix jerryxiao


## [0.2.11] - 2024-05-14

### Added

- Add some pixiv commands

- Add info command


### Changed

- Update

- Update actions

- Update

- Update

- Update

- Bump deps

- Update mentions for jerryxiao

- Reorganize files

- Update

- Update

- Update

- Update

- Update Actions

- 0.2.11


### Removed

- Remove rustls support


## [0.2.10] - 2024-05-10

### Added

- Add sticker upload command

- Add notification for sticker upload

- Add room replacement support

- Add formatting to JerryXiao commands

- Add editor configs

- Add admin room support

- Add ignore and unignore command

- Add pixiv command


### Changed

- Update reminder command

- Update

- Make some config optional

- Update

- Tweak tracing level

- Update JerryXiao function

- Refactor code, stage 1

- Refactor code, stage 2

- Refactor code, stage 3

- Update docs

- Update error reporting function

- Update

- Update

- Update

- Update

- Bump matrix-rust-sdk

- Bump dependencies

- Move to thiserror

- Update Cargo.toml

- Move to workspace

- Change sticker usage type to exact type

- Update

- Reorganize subcrates to subdir

- Bump deps

- Move to pixrs

- Move away from workspaces

- Allow using rustls

- 0.2.10


### Removed

- Remove dicer


## [0.2.9] - 2024-03-25

### Added

- Add quote command

- Add more HTML tags to ignore


### Changed

- Disable message backup

- Use UNIX shell word spliting

- Tweak loglevel

- Update command types

- 0.2.9


## [0.2.8] - 2024-03-24

### Changed

- Move session code to a new file

- Update

- Include a license

- Move Pages docs to mdBook docs

- Update

- Update

- Update

- 0.2.8


### Fixed

- Fix Actions


## [0.2.7] - 2024-03-24

### Added

- Add presence info

- Add remind command


### Changed

- Update typing functions

- Change ping delta formatting

- Bump matrix-rust-sdk

- Bump deps

- 0.2.7


## [0.2.6] - 2024-02-14

### Changed

- Update

- 0.2.6


## [0.2.5] - 2024-02-14

### Changed

- Update metas

- 0.2.5


## [0.2.4] - 2024-02-14

### Added

- Add typing notices

- Add At-Nahida prefix message handlers


### Changed

- Update

- Update

- 0.2.4


### Fixed

- Fix Hitokoto API


## [0.2.3] - 2024-02-08

### Added

- Add Hitokoto API


### Changed

- Update

- Bump matrix-rust-sdk

- Bump dependencies

- Update Rustdoc workflow

- Update

- 0.2.3


## [0.2.2] - 2023-12-03

### Added

- Add additional mentions for JerryXiao


### Changed

- Update jerryxiao handling

- Bump deps

- 0.2.2


### Fixed

- Fix retrying


## [0.2.1] - 2023-11-20

### Added

- Add instrument macros

- Add optional features

- Add SQLite to Nix environment


### Changed

- Move to simpler image size detection

- Update Cargo.toml

- Allow tracing to report errors

- Allow proper retrys

- 0.2.1


## [0.2.0] - 2023-11-17

### Added

- Add docs

- Add Pages

- Add autojoin


### Changed

- Update

- Optimize dicer output

- Give nom_error_message the expr string

- Graceful shutdown

- Update member changes command

- Update

- Update tracing level

- Reformat imports

- Update

- Update

- Update

- Update

- Update

- Update

- Update error handling

- Update

- Restructure the code

- 0.2.0


### Fixed

- Fix format

- Fix error reporting

- Fix workflow

- Fix logic


### Removed

- Remove extra dots


## [0.1.2] - 2023-11-14

### Added

- Add at symbols

- Add dicer implmentation


### Changed

- Update

- Revert "Add randomdraw"

This reverts commit 6a42ed86f369731f4dd9ed0f16396f4200da43ac.

- Retry randomdraw

- Update

- Update

- Update

- Update

- 0.1.2


## [0.1.1] - 2023-11-12

### Added

- Add name_changes command

- Add Jerryxiao function

- Add randomdraw

- Add divergence command

- Support proper intentional mentions for jerryxiao

- Add ignore and unignore command

- Add thiserror


### Changed

- Initial commit

- Update

- Update

- Update

- Update

- Bump to git version of Matrix Rust SDK

- Move message sending to outer level

- Update randomdraw implmentation

- Restructure the code

- Format error messages nicely

- 0.1.1


### Fixed

- Fix matrix.to link creation

- Fix jerryxiao message output

- Fix divergence command

- Fix randomdraw


### Removed

- Remove randomdraw

Can't make it work. :(


[0.3.4]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.3..v0.3.4
[0.3.3]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.2..v0.3.3
[0.3.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.1..v0.3.2
[0.3.1]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.0..v0.3.1
[0.3.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.11..v0.3.0
[0.2.11]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.10..v0.2.11
[0.2.10]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.9..v0.2.10
[0.2.9]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.8..v0.2.9
[0.2.8]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.7..v0.2.8
[0.2.7]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.6..v0.2.7
[0.2.6]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.5..v0.2.6
[0.2.5]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.4..v0.2.5
[0.2.4]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.3..v0.2.4
[0.2.3]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.2..v0.2.3
[0.2.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.1..v0.2.2
[0.2.1]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.0..v0.2.1
[0.2.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.1.2..v0.2.0
[0.1.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.1.1..v0.1.2

<!-- generated by git-cliff -->
