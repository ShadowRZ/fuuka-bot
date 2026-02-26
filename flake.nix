{
  description = "Fuuka Bot";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };

    flake-utils = {
      url = "github:numtide/flake-utils";
    };

    crane = {
      url = "github:ipetkov/crane";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
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
    {
      self,
      nixpkgs,
      crane,
      advisory-db,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        lib = pkgs.lib;
        toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        #src = craneLib.cleanCargoSource ./.;
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources ./.)
            # Also keep any markdown files
            (lib.fileset.fileFilter (file: file.hasExt "jinja") ./.)
            # Keep GraphQL queries
            (lib.fileset.fileFilter (file: file.hasExt "graphql") ./.)
          ];
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

        # Build *just* the cargo dependencies (of the entire workspace),
        # so we can reuse all of that work (e.g. via cachix) when running in CI
        # It is *highly* recommended to use something like cargo-hakari to avoid
        # cache misses when building individual top-level-crates
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        fuuka-bot = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            doCheck = false;
            cargoExtraArgs = "-p fuuka-bot";
          }
        );
      in
      {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit fuuka-bot;

          # Run clippy (and deny all warnings) on the crate source,
          # again, reusing the dependency artifacts from above.
          #
          # Note that this is done as a separate derivation so that
          # we can block the CI if there are issues here, but not
          # prevent downstream consumers from building our crate by itself.
          fuuka-bot-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          fuuka-bot-doc = craneLib.cargoDoc (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );

          # Check formatting
          fuuka-bot-fmt = craneLib.cargoFmt {
            inherit src;
          };

          # Audit dependencies
          fuuka-bot-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          fuuka-bot-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            # taplo arguments can be further customized below as needed
            # taploExtraArgs = "--config ./taplo.toml";
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `my-crate` if you do not want
          # the tests to run twice
          fuuka-bot-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
              cargoNextestPartitionsExtraArgs = "--no-tests=pass";
            }
          );
        };

        packages = {
          default = fuuka-bot;
        };

        devShells = {
          default = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            packages = [
              pkgs.just
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };
          aarch64-unknown-linux-musl = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            packages = [
              pkgs.pkgsCross.aarch64-multiplatform-musl.stdenv.cc
            ];
          };
        };
      }
    );
}
