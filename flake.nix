{
  description = "Fuuka Bot";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };
    crane = {
      url = "github:ipetkov/crane";
    };

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } (
      { inputs, ... }:
      {
        systems = [
          "x86_64-linux"
          "aarch64-linux"
        ];
        perSystem =
          {
            pkgs,
            self',
            inputs',
            ...
          }:
          (
            let
              crane = inputs.crane;
              advisory-db = inputs.advisory-db;
              fenix = inputs'.fenix.packages;

              inherit (pkgs) lib;

              craneLib = (crane.mkLib pkgs).overrideToolchain (_: fenix.stable.toolchain);
              graphQlFilter = path: _type: null != builtins.match ".*graphql$" path;
              graphQlOrCargo = path: type: (graphQlFilter path type) || (craneLib.filterCargoSources path type);

              src = lib.cleanSourceWith {
                src = ./.; # The original, unfiltered source
                filter = graphQlOrCargo;
                name = "source"; # Be reproducible, regardless of the directory name
              };

              # Common arguments can be set here to avoid repeating them later
              commonArgs = {
                inherit src;
                strictDeps = true;

                nativeBuildInputs = [
                  pkgs.pkg-config
                ];

                buildInputs = [
                  # Add additional build inputs here
                  pkgs.openssl
                  pkgs.sqlite
                ];
              };

              individualCrateArgs = commonArgs // {
                inherit cargoArtifacts;
                inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
                # NB: we disable tests since we'll run them all via cargo-nextest
                doCheck = false;
              };

              # Build *just* the cargo dependencies (of the entire workspace),
              # so we can reuse all of that work (e.g. via cachix) when running in CI
              # It is *highly* recommended to use something like cargo-hakari to avoid
              # cache misses when building individual top-level-crates
              cargoArtifacts = craneLib.buildDepsOnly commonArgs;

              fuuka-bot = craneLib.buildPackage (
                individualCrateArgs
                // {
                  pname = "fuuka-bot";
                  cargoExtraArgs = "-p fuuka-bot";
                  src = lib.fileset.toSource {
                    root = ./.;
                    fileset = lib.fileset.unions [
                      ./Cargo.toml
                      ./Cargo.lock
                      ./fuuka-bot
                      ./graphql
                    ];
                  };
                }
              );
            in
            {
              checks = {
                # Build the crate as part of `nix flake check` for convenience
                inherit fuuka-bot;

                # Check formatting
                fuuka-bot-fmt = craneLib.cargoFmt {
                  inherit src;
                };

                # Audit dependencies
                fuuka-bot-audit = craneLib.cargoAudit {
                  inherit src advisory-db;
                };
              };

              packages = {
                default = fuuka-bot;
              };

              devShells = {
                default = craneLib.devShell {
                  # Inherit inputs from checks.
                  inherit (self') checks;

                  # Extra inputs can be added here; cargo and rustc are provided by default.
                  packages = [
                    fenix.stable.rustfmt
                    fenix.stable.clippy
                  ];

                  RUST_SRC_PATH = "${fenix.stable.rust-src}/lib/rustlib/src/rust/library";
                };
              };
            }
          );
      }
    );
}
