# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-10-21

### Added
- *(bot)* Add versions command by @ShadowRZ
- *(nixpkgs)* Address clippy warnings by @ShadowRZ
- *(bot)* Add at command by @ShadowRZ
- *(ci)* Add debug info to nix-fast-build by @ShadowRZ
- *(tls)* Add matrix-sdk's rustls-aws-lc-rs feature as a default feature by @ShadowRZ
- *(flake)* Add cargo-deny to Nix Flake devShell by @ShadowRZ
- *(pr_tracker)* Add further debugging by @ShadowRZ
- Add back PR Tracker by @ShadowRZ
- Add ratelimit to Octocrab client by @ShadowRZ
- *(ci)* Add audit config by @ShadowRZ
- Add server command by @ShadowRZ
- Add timeout configuration by @ShadowRZ

### Changed
- *(deps)* Update and cleanup dependencies by @ShadowRZ
- *(server)* Change server version query to behave more like spec by @ShadowRZ
- *(pr_tracker)* Don't use GraphQL, send directly by @ShadowRZ
- *(deps)* Lock file maintenance by @renovate[bot] in [#168](https://github.com/ShadowRZ/fuuka-bot/pull/168)
- *(deps)* Update rust to v1.99.1 by @renovate[bot] in [#167](https://github.com/ShadowRZ/fuuka-bot/pull/167)
- *(deps)* Update rust to v1.99.0 by @renovate[bot] in [#158](https://github.com/ShadowRZ/fuuka-bot/pull/158)
- *(deps)* Lock file maintenance by @renovate[bot] in [#161](https://github.com/ShadowRZ/fuuka-bot/pull/161)
- *(deps)* Update rust crate minijinja to v3 by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk to 8769f1ab4e7b5f4e4a89089370ff472f89e1d1c3 by @ShadowRZ
- *(deps)* Update rust crate serde_json to 1.0.154 by @renovate[bot] in [#163](https://github.com/ShadowRZ/fuuka-bot/pull/163)
- *(bot)* Clarify at command's output by @ShadowRZ
- *(deps)* Lock file maintenance by @renovate[bot] in [#154](https://github.com/ShadowRZ/fuuka-bot/pull/154)
- *(deps)* Update rust crate thiserror to 2.0.21 by @renovate[bot] in [#153](https://github.com/ShadowRZ/fuuka-bot/pull/153)
- Update TOML formatting by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(deps)* Update orhun/git-cliff-action action to v4.9.1 by @renovate[bot] in [#152](https://github.com/ShadowRZ/fuuka-bot/pull/152)
- *(deps)* Lock file maintenance by @renovate[bot] in [#151](https://github.com/ShadowRZ/fuuka-bot/pull/151)
- *(deps)* Lock file maintenance by @renovate[bot] in [#147](https://github.com/ShadowRZ/fuuka-bot/pull/147)
- *(deps)* Update rust to v1.98.1 by @renovate[bot] in [#143](https://github.com/ShadowRZ/fuuka-bot/pull/143)
- *(deps)* Lock file maintenance by @renovate[bot] in [#144](https://github.com/ShadowRZ/fuuka-bot/pull/144)
- *(deps)* Update cachix/cachix-action digest to 38b0826 by @renovate[bot] in [#140](https://github.com/ShadowRZ/fuuka-bot/pull/140)
- *(deps)* Update orhun/git-cliff-action action to v4.9.0 by @renovate[bot] in [#141](https://github.com/ShadowRZ/fuuka-bot/pull/141)
- *(deps)* Update actions/deploy-pages action to v5.0.1 by @renovate[bot] in [#142](https://github.com/ShadowRZ/fuuka-bot/pull/142)
- *(deps)* Update rust to v1.98.0 by @renovate[bot] in [#130](https://github.com/ShadowRZ/fuuka-bot/pull/130)
- *(deps)* Lock file maintenance by @renovate[bot] in [#139](https://github.com/ShadowRZ/fuuka-bot/pull/139)
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(deps)* Update rust crates versions by @ShadowRZ
- *(deps)* Update rust crate thiserror to 2.0.20 by @renovate[bot] in [#136](https://github.com/ShadowRZ/fuuka-bot/pull/136)
- *(deps)* Update rust crate serde to 1.0.229 by @renovate[bot] in [#134](https://github.com/ShadowRZ/fuuka-bot/pull/134)
- *(deps)* Update rust crate http-body-util to 0.1.5 by @renovate[bot] in [#133](https://github.com/ShadowRZ/fuuka-bot/pull/133)
- *(deps)* Update rust crate bytes to 1.12.1 by @renovate[bot] in [#131](https://github.com/ShadowRZ/fuuka-bot/pull/131)
- *(deps)* Update rust crate http to 1.5.0 by @renovate[bot] in [#132](https://github.com/ShadowRZ/fuuka-bot/pull/132)
- *(deps)* Lock file maintenance by @renovate[bot] in [#129](https://github.com/ShadowRZ/fuuka-bot/pull/129)
- *(check)* Update deny checks by @ShadowRZ
- *(deps)* Bump dependency range for Renovate by @ShadowRZ
- *(deps)* Update rust crate chacha20 to 0.10.2 by @ShadowRZ
- *(ci)* Disable bitmaps related audits by @ShadowRZ
- *(deps)* Update dependencies by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(deps)* Lock file maintenance by @renovate[bot] in [#125](https://github.com/ShadowRZ/fuuka-bot/pull/125)
- *(deps)* Update rust crate minijinja to v2.23.0 by @renovate[bot] in [#124](https://github.com/ShadowRZ/fuuka-bot/pull/124)
- *(deps)* Update rust crate clap to v4.6.6 by @renovate[bot] in [#123](https://github.com/ShadowRZ/fuuka-bot/pull/123)
- *(deps)* Update rust crate minijinja to v2.22.0 by @renovate[bot] in [#122](https://github.com/ShadowRZ/fuuka-bot/pull/122)
- *(deps)* Lock file maintenance by @renovate[bot] in [#121](https://github.com/ShadowRZ/fuuka-bot/pull/121)
- *(deps)* Update rust crate http to v1.5.0 by @renovate[bot] in [#120](https://github.com/ShadowRZ/fuuka-bot/pull/120)
- *(deps)* Update rust to v1.97.1 by @renovate[bot] in [#94](https://github.com/ShadowRZ/fuuka-bot/pull/94)
- *(deps)* Lock file maintenance by @renovate[bot] in [#117](https://github.com/ShadowRZ/fuuka-bot/pull/117)
- *(deps)* Update rust crate octocrab to v0.54.1 by @renovate[bot] in [#118](https://github.com/ShadowRZ/fuuka-bot/pull/118)
- *(deps)* Update actions/checkout action to v7.0.1 by @renovate[bot] in [#114](https://github.com/ShadowRZ/fuuka-bot/pull/114)
- *(deps)* Update rust crate fastrand to v2.5.0 by @renovate[bot] in [#109](https://github.com/ShadowRZ/fuuka-bot/pull/109)
- *(deps)* Update rust crate formatx to v0.4.2 by @renovate[bot] in [#110](https://github.com/ShadowRZ/fuuka-bot/pull/110)
- *(deps)* Update rust crate serde_json to v1.0.151 by @renovate[bot] in [#111](https://github.com/ShadowRZ/fuuka-bot/pull/111)
- *(deps)* Update rust crate time to v0.3.54 by @renovate[bot] in [#112](https://github.com/ShadowRZ/fuuka-bot/pull/112)
- *(deps)* Update rust crate tokio to v1.53.1 by @renovate[bot] in [#115](https://github.com/ShadowRZ/fuuka-bot/pull/115)
- *(deps)* Update rust crate serde-bool to v0.1.5 by @renovate[bot] in [#116](https://github.com/ShadowRZ/fuuka-bot/pull/116)
- *(deps)* Update rust crate clap to v4.6.4 by @renovate[bot] in [#113](https://github.com/ShadowRZ/fuuka-bot/pull/113)
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(deps)* Update rust crate thiserror to v2.0.19 by @renovate[bot] in [#108](https://github.com/ShadowRZ/fuuka-bot/pull/108)
- *(deps)* Update rust crate serde to v1.0.229 by @renovate[bot] in [#107](https://github.com/ShadowRZ/fuuka-bot/pull/107)
- *(deps)* Update rust crate anyhow to v1.0.104 by @renovate[bot] in [#106](https://github.com/ShadowRZ/fuuka-bot/pull/106)
- *(deps)* Update rust-futures monorepo to v0.3.33 by @renovate[bot] in [#105](https://github.com/ShadowRZ/fuuka-bot/pull/105)
- *(deps)* Update rust crate tokio to v1.53.0 by @renovate[bot] in [#104](https://github.com/ShadowRZ/fuuka-bot/pull/104)
- *(deps)* Update rust crate tokio to v1.52.4 by @renovate[bot] in [#103](https://github.com/ShadowRZ/fuuka-bot/pull/103)
- *(deps)* Update rust crate toml to v1.1.3 by @renovate[bot] in [#100](https://github.com/ShadowRZ/fuuka-bot/pull/100)
- *(deps)* Update rust crate http-body-util to v0.1.4 by @renovate[bot] in [#98](https://github.com/ShadowRZ/fuuka-bot/pull/98)
- *(deps)* Update rust crate http-body to v1.1.0 by @renovate[bot] in [#99](https://github.com/ShadowRZ/fuuka-bot/pull/99)
- *(deps)* Update rust crate clap to v4.6.2 by @renovate[bot] in [#101](https://github.com/ShadowRZ/fuuka-bot/pull/101)
- *(deps)* Update rust crate regex to v1.13.1 by @renovate[bot] in [#102](https://github.com/ShadowRZ/fuuka-bot/pull/102)
- *(deps)* Lock file maintenance by @renovate[bot] in [#97](https://github.com/ShadowRZ/fuuka-bot/pull/97)
- *(deps)* Update rust crate regex to v1.13.0 by @renovate[bot] in [#95](https://github.com/ShadowRZ/fuuka-bot/pull/95)
- *(deps)* Update rust crate bytes to v1.12.1 by @renovate[bot] in [#93](https://github.com/ShadowRZ/fuuka-bot/pull/93)
- *(deps)* Lock file maintenance by @renovate[bot] in [#90](https://github.com/ShadowRZ/fuuka-bot/pull/90)
- *(deps)* Lock file maintenance by @renovate[bot] in [#89](https://github.com/ShadowRZ/fuuka-bot/pull/89)
- *(deps)* Update rust to v1.96.1 by @renovate[bot] in [#87](https://github.com/ShadowRZ/fuuka-bot/pull/87)
- *(deps)* Lock file maintenance by @renovate[bot] in [#88](https://github.com/ShadowRZ/fuuka-bot/pull/88)
- *(deps)* Lock file maintenance by @renovate[bot] in [#75](https://github.com/ShadowRZ/fuuka-bot/pull/75)
- *(deps)* Update rust crate formatx to 0.3.0 by @ShadowRZ
- *(deps)* Update rust crate quinn to 0.11.11 by @ShadowRZ
- *(deps)* Update matrix-sdk by @ShadowRZ
- *(deps)* Update rust crate time to v0.3.51 by @renovate[bot] in [#86](https://github.com/ShadowRZ/fuuka-bot/pull/86)
- *(deps)* Update rust crate bytes to v1.12.0 by @renovate[bot] in [#85](https://github.com/ShadowRZ/fuuka-bot/pull/85)
- *(deps)* Update rust crate minijinja to v2.21.0 by @renovate[bot] in [#82](https://github.com/ShadowRZ/fuuka-bot/pull/82)
- *(deps)* Update nixbuild/nix-quick-install-action action to v35 by @renovate[bot] in [#84](https://github.com/ShadowRZ/fuuka-bot/pull/84)
- *(deps)* Update actions/checkout action to v7 by @renovate[bot] in [#83](https://github.com/ShadowRZ/fuuka-bot/pull/83)
- *(deps)* Update rust crate time to v0.3.49 by @renovate[bot] in [#79](https://github.com/ShadowRZ/fuuka-bot/pull/79)
- *(deps)* Update rust crate octocrab to v0.53.1 by @renovate[bot] in [#78](https://github.com/ShadowRZ/fuuka-bot/pull/78)
- *(deps)* Update rust crate regex to v1.12.4 by @renovate[bot] in [#77](https://github.com/ShadowRZ/fuuka-bot/pull/77)
- *(deps)* Update rust crate http to v1.4.2 by @renovate[bot] in [#76](https://github.com/ShadowRZ/fuuka-bot/pull/76)
- *(deps)* Update actions/checkout action to v6.0.3 by @renovate[bot] in [#72](https://github.com/ShadowRZ/fuuka-bot/pull/72)
- *(deps)* Update rust crate rpassword to v7.5.4 by @renovate[bot] in [#71](https://github.com/ShadowRZ/fuuka-bot/pull/71)
- *(deps)* Update rust to v1.96.0 by @renovate[bot] in [#70](https://github.com/ShadowRZ/fuuka-bot/pull/70)
- *(deps)* Lock file maintenance by @renovate[bot] in [#67](https://github.com/ShadowRZ/fuuka-bot/pull/67)
- *(deps)* Update rust crate http to v1.4.1 by @renovate[bot] in [#69](https://github.com/ShadowRZ/fuuka-bot/pull/69)
- *(deps)* Update rust crate reqwest to v0.13.4 by @renovate[bot] in [#68](https://github.com/ShadowRZ/fuuka-bot/pull/68)
- *(deps)* Update rust crate tower-http to v0.6.11 by @renovate[bot] in [#63](https://github.com/ShadowRZ/fuuka-bot/pull/63)
- *(deps)* Lock file maintenance by @renovate[bot] in [#66](https://github.com/ShadowRZ/fuuka-bot/pull/66)
- *(deps)* Update rust crate serde_json to v1.0.150 by @renovate[bot] in [#65](https://github.com/ShadowRZ/fuuka-bot/pull/65)
- *(deps)* Update rust crate minijinja to v2.20.0 by @renovate[bot] in [#64](https://github.com/ShadowRZ/fuuka-bot/pull/64)
- *(deps)* Lock file maintenance by @renovate[bot] in [#61](https://github.com/ShadowRZ/fuuka-bot/pull/61)
- *(deps)* Update cachix/cachix-action digest to 5f2d7c5 by @renovate[bot] in [#58](https://github.com/ShadowRZ/fuuka-bot/pull/58)
- *(deps)* Update rust crate tokio to v1.52.3 by @renovate[bot] in [#60](https://github.com/ShadowRZ/fuuka-bot/pull/60)
- *(deps)* Lock file maintenance by @renovate[bot] in [#59](https://github.com/ShadowRZ/fuuka-bot/pull/59)
- *(deps)* Pin dependencies by @renovate[bot] in [#57](https://github.com/ShadowRZ/fuuka-bot/pull/57)
- *(ci)* Use nix-fast-build instead by @ShadowRZ
- *(deps)* Reconfigure Renovate by @ShadowRZ
- *(deps)* Flake.lock: Update by @ShadowRZ
- *(build)* Bump rust toolchain to 1.95.0 by @ShadowRZ
- *(flake)* Override stdenv for aarch64-unknown-linux-musl devShell by @ShadowRZ
- *(ci)* Use GHA for CI instead of Garnix by @ShadowRZ
- *(deps)* Update dependencies by @ShadowRZ
- *(deps)* Bump rust crate matrix-sdk by @ShadowRZ
- *(deps)* Update rust crate axum to v0.8.9 by @renovate[bot] in [#48](https://github.com/ShadowRZ/fuuka-bot/pull/48)
- *(deps)* Update rust crate clap to v4.6.1 by @renovate[bot] in [#49](https://github.com/ShadowRZ/fuuka-bot/pull/49)
- *(deps)* Update orhun/git-cliff-action action to v4.8.0 by @renovate[bot] in [#51](https://github.com/ShadowRZ/fuuka-bot/pull/51)
- *(deps)* Update rust crate reqwest to v0.13.3 by @renovate[bot] in [#52](https://github.com/ShadowRZ/fuuka-bot/pull/52)
- *(deps)* Update rust crate rpassword to v7.5.2 by @renovate[bot] in [#53](https://github.com/ShadowRZ/fuuka-bot/pull/53)
- *(deps)* Update rust crate tokio to v1.52.2 by @renovate[bot] in [#54](https://github.com/ShadowRZ/fuuka-bot/pull/54)
- *(deps)* Update rust crate tower-http to v0.6.10 by @renovate[bot] in [#56](https://github.com/ShadowRZ/fuuka-bot/pull/56)
- *(deps)* Update actions/upload-pages-artifact action to v5 by @renovate[bot] in [#45](https://github.com/ShadowRZ/fuuka-bot/pull/45)
- *(deps)* Update softprops/action-gh-release action to v3 by @renovate[bot] in [#47](https://github.com/ShadowRZ/fuuka-bot/pull/47)
- *(deps)* Update rust crate tokio to v1.52.1 by @renovate[bot] in [#44](https://github.com/ShadowRZ/fuuka-bot/pull/44)
- *(tls)* Ring -> aws-lc-rs by @ShadowRZ
- *(deps)* Update rust crate fastrand to v2.4.1 by @renovate[bot] in [#41](https://github.com/ShadowRZ/fuuka-bot/pull/41)
- *(deps)* Update rust crate octocrab to v0.49.7 by @renovate[bot] in [#40](https://github.com/ShadowRZ/fuuka-bot/pull/40)
- *(deps)* Update rust crate tokio to v1.51.0 by @renovate[bot] in [#30](https://github.com/ShadowRZ/fuuka-bot/pull/30)
- *(deps)* Bump rust crate matrix-sdk by @ShadowRZ
- Revert "fix(deps): update rust crate sha2 to 0.11.0 " by @ShadowRZ
- *(build)* Bump rust toolchain to 1.94.1 by @ShadowRZ
- Update by @ShadowRZ
- *(deps)* Update rust crate minijinja to v2.19.0 by @renovate[bot] in [#31](https://github.com/ShadowRZ/fuuka-bot/pull/31)
- *(deps)* Update rust crate clap to v4.6.0 by @renovate[bot] in [#32](https://github.com/ShadowRZ/fuuka-bot/pull/32)
- *(deps)* Update rust crate tracing-subscriber to v0.3.23 by @renovate[bot] in [#33](https://github.com/ShadowRZ/fuuka-bot/pull/33)
- *(deps)* Update softprops/action-gh-release action to v2.6.1 by @renovate[bot] in [#34](https://github.com/ShadowRZ/fuuka-bot/pull/34)
- *(deps)* Update actions/configure-pages action to v6 by @renovate[bot] in [#37](https://github.com/ShadowRZ/fuuka-bot/pull/37)
- *(deps)* Update actions/deploy-pages action to v5 by @renovate[bot] in [#38](https://github.com/ShadowRZ/fuuka-bot/pull/38)
- *(deps)* Update rust crate octocrab to v0.49.6 by @renovate[bot] in [#35](https://github.com/ShadowRZ/fuuka-bot/pull/35)
- *(deps)* Bump rust crate matrix-sdk by @ShadowRZ
- *(build)* Bump rust toolchain to 1.94.0 by @ShadowRZ
- Update by @ShadowRZ
- *(deps)* Update rust crate pin-project-lite to v0.2.17 by @renovate[bot] in [#29](https://github.com/ShadowRZ/fuuka-bot/pull/29)
- Simplify service types by @ShadowRZ
- *(pixiv)* Use a new pixiv ajax API crate by @ShadowRZ
- *(deps)* Bump Cargo.lock by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- Update by @ShadowRZ
- Move to Cargo workspaces by @ShadowRZ
- *(jerryxiao)* Use formatx for formatted content by @ShadowRZ
- Fixup error type formatting by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(crates)* Use tower::Service based API for HTTP interactions by @ShadowRZ
- *(hitokoto)* Use tower::Service based API for HTTP interactions by @ShadowRZ
- *(github)* Use HttpCacheLayer from octocrab by @ShadowRZ
- *(hitokoto)* Fill full hitokoto response payload by @ShadowRZ
- Update tracing logs to include prefixs by @ShadowRZ
- *(nixpkgs)* Also send message on PR closed by @ShadowRZ
- *(nixpkgs)* Also send message on PR merged by @ShadowRZ
- Enable send queue by @ShadowRZ
- Use ServiceBuilder methods for rate limiting by @ShadowRZ
- Pin pixrs version by @ShadowRZ
- *(deps)* Update rust crate clap to v4.5.60 by @renovate[bot] in [#22](https://github.com/ShadowRZ/fuuka-bot/pull/22)
- *(deps)* Update rust crate anyhow to v1.0.102 by @renovate[bot] in [#26](https://github.com/ShadowRZ/fuuka-bot/pull/26)
- *(deps)* Update rust crate minijinja to v2.16.0 by @renovate[bot] in [#27](https://github.com/ShadowRZ/fuuka-bot/pull/27)
- *(deps)* Update orhun/git-cliff-action action to v4.7.1 by @renovate[bot] in [#28](https://github.com/ShadowRZ/fuuka-bot/pull/28)
- Stop Renovate from bumping ruma git hash by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- Use matrix-sdk's federation-api feature by @ShadowRZ
- Cleanup by @ShadowRZ
- *(deps)* Bump Cargo.lock by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- Ensure test code are gated with cfg directive by @ShadowRZ
- Rewrite Nixpkgs PR tracker without tracking feature by @ShadowRZ
- Move to new Context API by @ShadowRZ
- Prepare out for refactoring GitHub support by @ShadowRZ
- Prepare for using octocrab by @ShadowRZ
- Reorgnaize configs by @ShadowRZ
- *(docs)* Cleanup crate docs by @ShadowRZ
- Move membership stream to new path by @ShadowRZ
- *(deps)* Update rust crate toml to v0.9.12 by @ShadowRZ
- Relicense to AGPL-3.0-or-later by @ShadowRZ
- *(media-proxy)* Use matrix_sdk::Client directly instead by @ShadowRZ
- Move environment functions to separate module by @ShadowRZ
- Move Matrix functions to crate::matrix by @ShadowRZ
- *(deps)* Update rust crate thiserror to v2.0.18 by @ShadowRZ
- *(deps)* Bump Cargo.lock by @ShadowRZ
- *(deps)* Update rust crate reqwest to v0.12.28 by @ShadowRZ
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- Reformat code by @ShadowRZ
- Update by @ShadowRZ
- Use builder pattern for constructing client by @ShadowRZ
- *(deps)* Update rust crate anyhow to v1.0.101 by @renovate[bot] in [#20](https://github.com/ShadowRZ/fuuka-bot/pull/20)
- *(deps)* Update rust crate time to v0.3.47 by @renovate[bot] in [#19](https://github.com/ShadowRZ/fuuka-bot/pull/19)
- *(fmt)* Reformat codes by @ShadowRZ
- *(command)* Allow Pixiv ranking to pass category by @ShadowRZ
- Move bot command system to clap by @ShadowRZ
- *(cli)* Use clap for parsing CLI arguments by @ShadowRZ
- Move some matrix related functions to crate::matrix by @ShadowRZ
- *(deps)* Update rust crate regex to v1.12.3 by @renovate[bot] in [#18](https://github.com/ShadowRZ/fuuka-bot/pull/18)
- *(deps)* Update rust crate bytes to v1.11.1 by @renovate[bot] in [#17](https://github.com/ShadowRZ/fuuka-bot/pull/17)

### Fixed
- *(ci)* Correct GitHub release action (x3) by @ShadowRZ
- *(ci)* Correct GitHub release action (x2) by @ShadowRZ
- *(ci)* Correct GitHub release action by @ShadowRZ
- *(bot)* Correct span names by @ShadowRZ
- *(nixpkgs)* Move track option check outside by @ShadowRZ
- *(deps)* Update rust crate octocrab to 0.54.5 by @renovate[bot] in [#169](https://github.com/ShadowRZ/fuuka-bot/pull/169)
- *(deps)* Update rust crate minijinja to 3.0.1 by @renovate[bot] in [#160](https://github.com/ShadowRZ/fuuka-bot/pull/160)
- *(deps)* Update rust crate tokio to 1.53.3 by @renovate[bot] in [#166](https://github.com/ShadowRZ/fuuka-bot/pull/166)
- *(deps)* Update rust crate octocrab to 0.54.4 by @renovate[bot] in [#165](https://github.com/ShadowRZ/fuuka-bot/pull/165)
- *(deps)* Update rust crate sha2 to 0.11.1 by @renovate[bot] in [#164](https://github.com/ShadowRZ/fuuka-bot/pull/164)
- *(deps)* Update rust crate toml to 1.1.8 by @renovate[bot] in [#162](https://github.com/ShadowRZ/fuuka-bot/pull/162)
- *(deps)* Update rust crate octocrab to 0.54.3 by @renovate[bot] in [#159](https://github.com/ShadowRZ/fuuka-bot/pull/159)
- *(deps)* Update rust crate tokio to 1.53.2 by @renovate[bot] in [#156](https://github.com/ShadowRZ/fuuka-bot/pull/156)
- *(deps)* Update rust crate serde-bool to 0.1.6 by @renovate[bot] in [#155](https://github.com/ShadowRZ/fuuka-bot/pull/155)
- *(deps)* Update rust crate toml to 1.1.7 by @renovate[bot] in [#157](https://github.com/ShadowRZ/fuuka-bot/pull/157)
- *(config)* Move serde field rename to variant by @ShadowRZ
- *(bot)* Don't respawn send queue by @ShadowRZ
- *(deps)* Update rust crate clap to 4.6.7 by @renovate[bot] in [#150](https://github.com/ShadowRZ/fuuka-bot/pull/150)
- *(deps)* Update rust crate octocrab to 0.54.2 by @renovate[bot] in [#149](https://github.com/ShadowRZ/fuuka-bot/pull/149)
- *(deps)* Update rust crate crc32fast to 1.5.2 by @renovate[bot] in [#148](https://github.com/ShadowRZ/fuuka-bot/pull/148)
- *(deps)* Update rust crate toml to 1.1.6 by @renovate[bot] in [#146](https://github.com/ShadowRZ/fuuka-bot/pull/146)
- *(deps)* Update rust crate reqwest to 0.13.5 by @renovate[bot] in [#145](https://github.com/ShadowRZ/fuuka-bot/pull/145)
- *(ci)* Don't use nix-fast-build by @ShadowRZ
- *(ci)* Correct CI nix-fast-build command by @ShadowRZ
- *(matrix)* Fix federation OutgoingRequest import by @ShadowRZ
- *(deps)* Update rust crate imagesize to 0.15.0 by @renovate[bot] in [#96](https://github.com/ShadowRZ/fuuka-bot/pull/96)
- *(deps)* Update rust crate octocrab to 0.54.0 by @renovate[bot] in [#92](https://github.com/ShadowRZ/fuuka-bot/pull/92)
- *(deps)* Update rust crate formatx to 0.4.0 by @renovate[bot] in [#91](https://github.com/ShadowRZ/fuuka-bot/pull/91)
- *(audit)* Ignore RUSTSEC-2026-0173 for now by @ShadowRZ
- *(deps)* Update rust crate tower-http to 0.7.0 by @renovate[bot] in [#81](https://github.com/ShadowRZ/fuuka-bot/pull/81)
- *(deps)* Update rust crate octocrab to 0.53.0 by @renovate[bot] in [#74](https://github.com/ShadowRZ/fuuka-bot/pull/74)
- *(deps)* Update rust crate octocrab to 0.52.0 by @renovate[bot] in [#73](https://github.com/ShadowRZ/fuuka-bot/pull/73)
- *(deps)* Update rust crate octocrab to 0.51.0 by @renovate[bot] in [#62](https://github.com/ShadowRZ/fuuka-bot/pull/62)
- *(ci)* Actually setup Cachix in CI by @ShadowRZ
- Fix clippy lint by @ShadowRZ
- *(github)* Directly use respose type since octocrab handles errors now by @ShadowRZ
- *(send_queue)* Try to delegate send queue progress reporing to a new task by @ShadowRZ
- *(deps)* Update rust crate octocrab to 0.50.0 by @renovate[bot] in [#55](https://github.com/ShadowRZ/fuuka-bot/pull/55)
- *(deps)* Update rust crate rustls-webpki to 0.103.10 by @ShadowRZ
- *(deps)* Update rust crate sha2, hmac by @ShadowRZ
- *(deps)* Update rust crate toml to v1 by @renovate[bot] in [#23](https://github.com/ShadowRZ/fuuka-bot/pull/23)
- *(deps)* Update rust crate sha2 to 0.11.0 by @renovate[bot] in [#36](https://github.com/ShadowRZ/fuuka-bot/pull/36)
- *(deps)* Update rust crate file-format to 0.29.0 by @renovate[bot] in [#39](https://github.com/ShadowRZ/fuuka-bot/pull/39)
- *(config)* Allow default value for tag triggers by @ShadowRZ
- Fix example config by @ShadowRZ
- Fix log emitting by @ShadowRZ
- Fix serde defaults by @ShadowRZ

### Removed
- *(flake)* Remove flake-utils and flake-parts by @ShadowRZ
- Remove custom error type by @ShadowRZ
- Remove tokio-console and direct journald support by @ShadowRZ
- Remove hickory-dns support for reqwest by @ShadowRZ
- Remove rustls-tls-native-roots feature by @ShadowRZ
- Remove interactive-login feature by @ShadowRZ

## [0.4.7] - 2026-02-02

### Added
- *(command)* Add bot subcommands by @ShadowRZ
- *(command)* Add delete command by @ShadowRZ
- Add secret storage commands by @ShadowRZ
- Add recover-cross-signing subcommand by @ShadowRZ
- *(dev)* Add Justfile by @ShadowRZ
- *(build)* Add more targets by @ShadowRZ

### Changed
- Make default timeout a static variable by @ShadowRZ
- Increase Matrix client timeout to 5 minutes by @ShadowRZ
- *(build)* Bump rust toolchain to 1.93.0 by @ShadowRZ
- Update by @ShadowRZ
- *(deps)* Update rust crate time to v0.3.46 by @renovate[bot] in [#16](https://github.com/ShadowRZ/fuuka-bot/pull/16)
- *(deps)* Update rust crate matrix-sdk by @ShadowRZ
- *(deps)* Update rust crate thiserror to v2.0.18 by @renovate[bot] in [#15](https://github.com/ShadowRZ/fuuka-bot/pull/15)
- *(deps)* Update actions/checkout action to v6.0.2 by @renovate[bot] in [#14](https://github.com/ShadowRZ/fuuka-bot/pull/14)
- *(deps)* Update rust crate tokio to v1.49.0 by @renovate[bot] in [#11](https://github.com/ShadowRZ/fuuka-bot/pull/11)
- *(deps)* Update rust crate minijinja to v2.15.1 by @renovate[bot] in [#10](https://github.com/ShadowRZ/fuuka-bot/pull/10)
- *(deps)* Update tokio-tracing monorepo by @renovate[bot] in [#9](https://github.com/ShadowRZ/fuuka-bot/pull/9)
- *(deps)* Update rust crate url to v2.5.8 by @renovate[bot] in [#8](https://github.com/ShadowRZ/fuuka-bot/pull/8)
- *(deps)* Update rust crate toml to v0.9.11 by @renovate[bot] in [#7](https://github.com/ShadowRZ/fuuka-bot/pull/7)
- *(deps)* Update rust crate shell-words to v1.1.1 by @renovate[bot] in [#6](https://github.com/ShadowRZ/fuuka-bot/pull/6)
- *(deps)* Update rust crate serde_json to v1.0.149 by @renovate[bot] in [#5](https://github.com/ShadowRZ/fuuka-bot/pull/5)
- *(deps)* Update rust crate axum to v0.8.8 by @renovate[bot] in [#3](https://github.com/ShadowRZ/fuuka-bot/pull/3)
- *(ci)* Stop automatically updating changelogs on each push to master by @ShadowRZ
- *(ci)* Update CI configs by @ShadowRZ
- Configure Renovate by @ShadowRZ

### Fixed
- *(changelog)* Properly categorize some commits by @ShadowRZ
- Fetch backup status from server by @ShadowRZ
- *(deps)* Update rust crate graphql_client to 0.16.0 by @renovate[bot] in [#12](https://github.com/ShadowRZ/fuuka-bot/pull/12)
- *(docs)* Fix mdbook docs build by @ShadowRZ

### Removed
- Remove unnecessary `pub(self)` by @ShadowRZ

### New Contributors
* @renovate[bot] made their first contribution in [#16](https://github.com/ShadowRZ/fuuka-bot/pull/16)

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

### Added
- Add Tokio Console and Journal logging by @ShadowRZ
- Add BiliBili video extractor and command by @ShadowRZ

### Changed
- 0.4.3 by @ShadowRZ
- Optimize error reports from GitHub GraphQL response by @ShadowRZ
- Cleanup tracing by @ShadowRZ
- Update Nixpkgs PR tracking info template by @ShadowRZ
- Fix stylecheck violations by @ShadowRZ
- Overhaul tracing by @ShadowRZ
- Bump packages by @ShadowRZ
- Bump Rust version to 1.89.0 by @ShadowRZ
- Reformat by @ShadowRZ
- Allow Release action to be manually started by @ShadowRZ

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

### Added
- Add quote command by @ShadowRZ
- Add rooms command by @ShadowRZ
- Add back commands by @ShadowRZ

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
- Bump dependencies by @ShadowRZ
- Update by @ShadowRZ

### Fixed
- Fix quote function by @ShadowRZ
- Fix pixiv command by @ShadowRZ
- Use more accurate emoji for branch status by @ShadowRZ
- Fix JerryXiao by @ShadowRZ

## [0.4.0] - 2025-01-04

### Added
- Add direnv by @ShadowRZ
- Add more size optimizations by @ShadowRZ

### Changed
- 0.4.0 by @ShadowRZ
- Updated docs to reflect current commands and configs by @ShadowRZ
- Translated tags by @ShadowRZ
- Bump axum by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Cleanup by @ShadowRZ
- Reformat by @ShadowRZ
- Update by @ShadowRZ
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

### Removed
- Remove workspaces by @ShadowRZ

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

### Added
- Add cross signing management by @ShadowRZ
- Add nixpkgs command by @ShadowRZ
- Add proper recovery support by @ShadowRZ

### Changed
- 0.3.3 by @ShadowRZ
- Update flake builds by @ShadowRZ
- Update by @ShadowRZ
- Nixpkgs PR tracking by @ShadowRZ
- Use cynic to build GraphQL query by @ShadowRZ
- Move to workspace by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Ping-admin command by @ShadowRZ
- Update docs by @ShadowRZ
- Apply Clippy suggestions by @ShadowRZ
- Bump dependencies by @ShadowRZ
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
- Add changelog workflow by @ShadowRZ
- Add changelog by @ShadowRZ
- Add interactive-login feature by @ShadowRZ
- Add a warning to avatar_changes by @ShadowRZ
- *(pixiv)* Add www prefix to pixiv links by @Guanran928

### Changed
- 0.3.1 by @ShadowRZ
- Bump dependencies by @ShadowRZ
- Systemd service by @ShadowRZ
- Allow specifiy directories in env var by @ShadowRZ
- Set RUST_SRC_PATH in flakes by @ShadowRZ
- Update Flakes Rust toolchain by @ShadowRZ
- Use the same toolchain for two devshells by @ShadowRZ
- Update flakes by @ShadowRZ
- BiliBili extractor by @ShadowRZ
- Make pixiv command respect config by @ShadowRZ
- Migrate deprecated extension by @ShadowRZ
- *(flake)* Use crane instead by @ShadowRZ
- Update Dev Container by @ShadowRZ
- Merge pull request #1 from Guanran928/pixiv-url by @ShadowRZ in [#1](https://github.com/ShadowRZ/fuuka-bot/pull/1)

### Fixed
- Fix ping command by @ShadowRZ
- Fix Actions by @ShadowRZ
- Properly restart sync on error by @ShadowRZ
- *(pixiv)* Use /artworks/{id} instead of /i/{id} by @Guanran928

### Removed
- Remove Dev Containers by @ShadowRZ
- Remove quote by @ShadowRZ

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

[unreleased]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.7...v0.5.0
[0.4.7]: https://github.com/ShadowRZ/fuuka-bot/compare/v0.4.6...v0.4.7
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
