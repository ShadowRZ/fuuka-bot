# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- *(ci)* Update CI configs by @ShadowRZ
- Configure Renovate by @ShadowRZ

### Fixed
- *(docs)* Fix mdbook docs build by @ShadowRZ

## [0.4.6] - 2025-11-22

### Changed
- Also add tag message to changelog by @ShadowRZ
- Migrate BiliBili video formatting to minijinja by @ShadowRZ
- Migrate Pixiv illust formatting to minijinja by @ShadowRZ
- Bump all transitive dependencies by @ShadowRZ
- Bump matrix-rust-sdk by @ShadowRZ
- Pin all dependencies to exact versions by @ShadowRZ
- Bump console-subscriber to 0.5.0 by @ShadowRZ
- Bump bytes to 1.11.0 by @ShadowRZ
- Bump axum to 0.8.7 by @ShadowRZ

## [0.4.5] - 2025-11-19

### Fixed
- Allow starting the bot even if RUST_LOG isn't present by @ShadowRZ

## [0.4.4] - 2025-11-18

### Changed
- Pin all GitHub Action to version commits by @ShadowRZ
- Bump Cargo Crates dependencies by @ShadowRZ
- Bump Rust version to 1.91.0 by @ShadowRZ
- Use rust-overlay instead of fenix to manage Rust toolchain in Nix by @ShadowRZ
- Bump Cargo Crates dependencies by @ShadowRZ
- Bump Rust version to 1.90.0 by @ShadowRZ
- Update by @ShadowRZ
- Unify Pixiv content sending by @ShadowRZ
- Captioned Pixiv illust info by @ShadowRZ
- Cleanup by @ShadowRZ

### Fixed
- Don't override any RUST_LOG provided directives by @ShadowRZ

## [0.4.3] - 2025-08-26

### Changed
- 0.4.3 by @ShadowRZ
- Optimize error reports from GitHub GraphQL response by @ShadowRZ
- Cleanup tracing by @ShadowRZ
- Update Nixpkgs PR tracking info template by @ShadowRZ
- Add Tokio Console and Journal logging by @ShadowRZ
- Fix stylecheck violations by @ShadowRZ
- Overhaul tracing by @ShadowRZ
- Bump packages by @ShadowRZ
- Bump Rust version to 1.89.0 by @ShadowRZ
- Reformat by @ShadowRZ
- Allow Release action to be manually started by @ShadowRZ
- Add BiliBili video extractor and command by @ShadowRZ

### Fixed
- Only log errors on leaf span by @ShadowRZ
- Fix RoomExt::in_reply_to_event by @ShadowRZ
- Restrict ignore ability to admin user by @ShadowRZ
- Fix GitHub Actions by @ShadowRZ

## [0.4.2] - 2025-08-11

### Changed
- Replace TOML VS Code extension with tombi by @ShadowRZ
- Migrate to let chains and Rust 1.88.0 by @ShadowRZ

### Fixed
- Fix tombstone following logic to account for v12 rooms by @ShadowRZ

## [0.4.1] - 2025-08-06

### Changed
- 0.4.1 by @ShadowRZ
- Inline all format string variables by @ShadowRZ
- Bump Cargo packages by @ShadowRZ
- Bump Rust to 1.88.0 by @ShadowRZ
- Update by @ShadowRZ
- Bump Rust to 1.87.0 by @ShadowRZ
- Update by @ShadowRZ
- Bump Cargo packages by @ShadowRZ
- Put Rust toolchain conifguration to dedicated file by @ShadowRZ
- Restructure logging by @ShadowRZ
- Move Hitokoto APIs to crate::services::hikokoto by @ShadowRZ
- Bump Cargo packages by @ShadowRZ
- Update by @ShadowRZ
- Fix clippy warnings by @ShadowRZ
- Allow quote renderer process to spawn on a blocking thread by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Rewrite quote renderer to use parley instead by @ShadowRZ
- Reformat by @ShadowRZ
- Update Cargo.toml by @ShadowRZ
- Use watch channels by @ShadowRZ
- Refactor to use parking_lot atomics by @ShadowRZ
- Add quote command by @ShadowRZ
- Add rooms command by @ShadowRZ
- Reformat by @ShadowRZ
- Also add a profile to abort on panic by @ShadowRZ
- Use graphql_client by @ShadowRZ
- Rustls + Hickory DNS by @ShadowRZ
- Use manual dispatch instead by @ShadowRZ
- Bump packages by @ShadowRZ
- Update by @ShadowRZ
- Reuse reqwest::Client across libs by @ShadowRZ
- Bump Cargo.lock by @ShadowRZ
- Update pixiv fomatters + Reformat by @ShadowRZ
- Bump cargo packages by @ShadowRZ
- Opt in to Rust 2024 Edition by @ShadowRZ
- Bump cargo crates by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Add back commands by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Update by @ShadowRZ

### Fixed
- Fix quote function by @ShadowRZ
- Fix pixiv command by @ShadowRZ
- Use more accurate emoji for branch status by @ShadowRZ
- Fix JerryXiao by @ShadowRZ

## [0.4.0] - 2025-01-04

### Changed
- 0.4.0 by @ShadowRZ
- Updated docs to reflect current commands and configs by @ShadowRZ
- Translated tags by @ShadowRZ
- Bump axum by @ShadowRZ
- Add direnv by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Cleanup by @ShadowRZ
- Add more size optimizations by @ShadowRZ
- Reformat by @ShadowRZ
- Update by @ShadowRZ
- Remove workspaces by @ShadowRZ
- Cleanup by @ShadowRZ
- Move GraqhQL to gql_client by @ShadowRZ
- Bump by @ShadowRZ
- Update by @ShadowRZ
- Rewrite command system by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ

### Fixed
- Fix some warnings by @ShadowRZ
- Derive defaults for some config by @ShadowRZ
- Fix jerryxiao by @ShadowRZ
- Fix nixpkgs command by @ShadowRZ

## [0.3.4] - 2024-11-27

### Changed
- 0.3.4 by @ShadowRZ
- Redo some code by @ShadowRZ
- Update by @ShadowRZ
- Refactor Rust files by @ShadowRZ
- Import ruma from matrix_sdk by @ShadowRZ
- Use dptree based dispatching by @ShadowRZ
- Allow media token to expire by @ShadowRZ
- Media Proxy by @ShadowRZ

### Fixed
- Fix release action by @ShadowRZ

## [0.3.3] - 2024-11-05

### Changed
- 0.3.3 by @ShadowRZ
- Update flake builds by @ShadowRZ
- Update by @ShadowRZ
- Nixpkgs PR tracking by @ShadowRZ
- Use cynic to build GraphQL query by @ShadowRZ
- Move to workspace by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Add cross signing management by @ShadowRZ
- Add nixpkgs command by @ShadowRZ
- Ping-admin command by @ShadowRZ
- Update docs by @ShadowRZ
- Apply Clippy suggestions by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Add proper recovery support by @ShadowRZ
- Enable recovery instead by @ShadowRZ

### Fixed
- Fix sending done status by @ShadowRZ
- Fix user_id command by @ShadowRZ
- Fix code for changed matrix-rust-sdk API by @ShadowRZ
- Fix actions by @ShadowRZ

### New Contributors
* @github-actions[bot] made their first contribution

## [0.3.2] - 2024-09-20

### Changed
- 0.3.2 by @ShadowRZ
- Update warning in avatar_changes by @ShadowRZ
- Update actions by @ShadowRZ

## [0.3.1] - 2024-09-20

### Added
- *(pixiv)* Add www prefix to pixiv links by @Guanran928

### Changed
- 0.3.1 by @ShadowRZ
- Add changelog workflow by @ShadowRZ
- Add changelog by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Add interactive-login feature by @ShadowRZ
- Systemd service by @ShadowRZ
- Allow specifiy directories in env var by @ShadowRZ
- Set RUST_SRC_PATH in flakes by @ShadowRZ
- Update Flakes Rust toolchain by @ShadowRZ
- Remove Dev Containers by @ShadowRZ
- Use the same toolchain for two devshells by @ShadowRZ
- Update flakes by @ShadowRZ
- BiliBili extractor by @ShadowRZ
- Make pixiv command respect config by @ShadowRZ
- Add a warning to avatar_changes by @ShadowRZ
- Migrate deprecated extension by @ShadowRZ
- Remove quote by @ShadowRZ
- *(flake)* Use crane instead by @ShadowRZ
- Update Dev Container by @ShadowRZ
- Merge pull request #1 from Guanran928/pixiv-url by @ShadowRZ in [#1](https://github.com/ShadowRZ/fuuka-bot/pull/1)

### Fixed
- Fix ping command by @ShadowRZ
- Fix Actions by @ShadowRZ
- Properly restart sync on error by @ShadowRZ
- *(pixiv)* Use /artworks/{id} instead of /i/{id} by @Guanran928

### New Contributors
* @Guanran928 made their first contribution

## [0.3.0] - 2024-05-15

### Changed
- V0.3.0 by @ShadowRZ
- Fix jerryxiao by @ShadowRZ
- Fix title extractor by @ShadowRZ
- Update nahida functions by @ShadowRZ
- Update configuration docs by @ShadowRZ
- Update nahida functions by @ShadowRZ
- Move specials to reading config file by @ShadowRZ
- Refactor config by @ShadowRZ

## [0.2.11] - 2024-05-14

### Added
- Add info command by @ShadowRZ
- Add some pixiv commands by @ShadowRZ

### Changed
- 0.2.11 by @ShadowRZ
- Update Actions by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Reorganize files by @ShadowRZ
- Update mentions for jerryxiao by @ShadowRZ
- Bump deps by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update actions by @ShadowRZ
- Update by @ShadowRZ

### Removed
- Remove rustls support by @ShadowRZ

## [0.2.10] - 2024-05-10

### Added
- Add pixiv command by @ShadowRZ
- Add ignore and unignore command by @ShadowRZ
- Add admin room support by @ShadowRZ
- Add editor configs by @ShadowRZ
- Add formatting to JerryXiao commands by @ShadowRZ
- Add room replacement support by @ShadowRZ
- Add notification for sticker upload by @ShadowRZ
- Add sticker upload command by @ShadowRZ

### Changed
- 0.2.10 by @ShadowRZ
- Allow using rustls by @ShadowRZ
- Move away from workspaces by @ShadowRZ
- Move to pixrs by @ShadowRZ
- Bump deps by @ShadowRZ
- Reorganize subcrates to subdir by @ShadowRZ
- Update by @ShadowRZ
- Change sticker usage type to exact type by @ShadowRZ
- Move to workspace by @ShadowRZ
- Update Cargo.toml by @ShadowRZ
- Move to thiserror by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Bump matrix-rust-sdk by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update error reporting function by @ShadowRZ
- Update docs by @ShadowRZ
- Refactor code, stage 3 by @ShadowRZ
- Refactor code, stage 2 by @ShadowRZ
- Refactor code, stage 1 by @ShadowRZ
- Update JerryXiao function by @ShadowRZ
- Tweak tracing level by @ShadowRZ
- Update by @ShadowRZ
- Make some config optional by @ShadowRZ
- Update by @ShadowRZ
- Update reminder command by @ShadowRZ

### Removed
- Remove dicer by @ShadowRZ

## [0.2.9] - 2024-03-25

### Added
- Add more HTML tags to ignore by @ShadowRZ
- Add quote command by @ShadowRZ

### Changed
- 0.2.9 by @ShadowRZ
- Update command types by @ShadowRZ
- Tweak loglevel by @ShadowRZ
- Use UNIX shell word spliting by @ShadowRZ
- Disable message backup by @ShadowRZ

## [0.2.8] - 2024-03-24

### Changed
- 0.2.8 by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Fix Actions by @ShadowRZ
- Update by @ShadowRZ
- Move Pages docs to mdBook docs by @ShadowRZ
- Include a license by @ShadowRZ
- Update by @ShadowRZ
- Move session code to a new file by @ShadowRZ

## [0.2.7] - 2024-03-24

### Added
- Add remind command by @ShadowRZ
- Add presence info by @ShadowRZ

### Changed
- 0.2.7 by @ShadowRZ
- Bump deps by @ShadowRZ
- Bump matrix-rust-sdk by @ShadowRZ
- Change ping delta formatting by @ShadowRZ
- Update typing functions by @ShadowRZ

## [0.2.6] - 2024-02-14

### Changed
- 0.2.6 by @ShadowRZ
- Update by @ShadowRZ

## [0.2.5] - 2024-02-14

### Changed
- 0.2.5 by @ShadowRZ
- Update metas by @ShadowRZ

## [0.2.4] - 2024-02-14

### Added
- Add At-Nahida prefix message handlers by @ShadowRZ
- Add typing notices by @ShadowRZ

### Changed
- 0.2.4 by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Fix Hitokoto API by @ShadowRZ

## [0.2.3] - 2024-02-08

### Added
- Add Hitokoto API by @ShadowRZ

### Changed
- 0.2.3 by @ShadowRZ
- Update by @ShadowRZ
- Update Rustdoc workflow by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Bump matrix-rust-sdk by @ShadowRZ
- Update by @ShadowRZ

## [0.2.2] - 2023-12-03

### Added
- Add additional mentions for JerryXiao by @ShadowRZ

### Changed
- 0.2.2 by @ShadowRZ
- Bump deps by @ShadowRZ
- Fix retrying by @ShadowRZ
- Update jerryxiao handling by @ShadowRZ

## [0.2.1] - 2023-11-20

### Added
- Add SQLite to Nix environment by @ShadowRZ
- Add optional features by @ShadowRZ
- Add instrument macros by @ShadowRZ

### Changed
- 0.2.1 by @ShadowRZ
- Allow proper retrys by @ShadowRZ
- Allow tracing to report errors by @ShadowRZ
- Update Cargo.toml by @ShadowRZ
- Move to simpler image size detection by @ShadowRZ

## [0.2.0] - 2023-11-17

### Added
- Add autojoin by @ShadowRZ
- Add Pages by @ShadowRZ
- Add docs by @ShadowRZ

### Changed
- Fix logic by @ShadowRZ
- 0.2.0 by @ShadowRZ
- Restructure the code by @ShadowRZ
- Update by @ShadowRZ
- Update error handling by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Reformat imports by @ShadowRZ
- Update tracing level by @ShadowRZ
- Update by @ShadowRZ
- Update member changes command by @ShadowRZ
- Graceful shutdown by @ShadowRZ
- Fix workflow by @ShadowRZ
- Fix error reporting by @ShadowRZ
- Give nom_error_message the expr string by @ShadowRZ
- Optimize dicer output by @ShadowRZ
- Update by @ShadowRZ
- Fix format by @ShadowRZ

### Removed
- Remove extra dots by @ShadowRZ

## [0.1.2] - 2023-11-14

### Added
- Add dicer implmentation by @ShadowRZ
- Add at symbols by @ShadowRZ

### Changed
- 0.1.2 by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Retry randomdraw by @ShadowRZ
- Revert "Add randomdraw" by @ShadowRZ
- Update by @ShadowRZ

## [0.1.1] - 2023-11-12

### Added
- Add thiserror by @ShadowRZ
- Add ignore and unignore command by @ShadowRZ
- Support proper intentional mentions for jerryxiao by @ShadowRZ
- Add divergence command by @ShadowRZ
- Add randomdraw by @ShadowRZ
- Add Jerryxiao function by @ShadowRZ
- Add name_changes command by @ShadowRZ

### Changed
- 0.1.1 by @ShadowRZ
- Format error messages nicely by @ShadowRZ
- Restructure the code by @ShadowRZ
- Update randomdraw implmentation by @ShadowRZ
- Fix randomdraw by @ShadowRZ
- Fix divergence command by @ShadowRZ
- Move message sending to outer level by @ShadowRZ
- Bump to git version of Matrix Rust SDK by @ShadowRZ
- Fix jerryxiao message output by @ShadowRZ
- Update by @ShadowRZ
- Fix matrix.to link creation by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Update by @ShadowRZ
- Initial commit by @ShadowRZ

### Removed
- Remove randomdraw by @ShadowRZ

### New Contributors
* @ShadowRZ made their first contribution

[unreleased]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.6...HEAD
[0.4.6]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.5...v0.4.6
[0.4.5]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.4...v0.4.5
[0.4.4]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.2...v0.4.3
[0.4.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.1...v0.4.2
[0.4.1]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.4...v0.4.0
[0.3.4]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.11...v0.3.0
[0.2.11]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.10...v0.2.11
[0.2.10]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.9...v0.2.10
[0.2.9]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.8...v0.2.9
[0.2.8]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.7...v0.2.8
[0.2.7]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.6...v0.2.7
[0.2.6]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.5...v0.2.6
[0.2.5]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.4...v0.2.5
[0.2.4]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.1.1...v0.1.2

<!-- generated by git-cliff -->
