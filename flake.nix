{
  description = "Fuuka Bot";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";

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
      fenix,
      flake-utils,
      advisory-db,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

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

        craneLibLLvmTools = craneLib.overrideToolchain (fenix.packages.${system}.stable.toolchain);

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        fuuka-bot = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
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

          # Audit licenses
          fuuka-bot-deny = craneLib.cargoDeny {
            inherit src;
          };

          # Run tests with cargo-nextest
          # Consider setting `doCheck = false` on `fuuka-bot` if you do not want
          # the tests to run twice
          fuuka-bot-nextest = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
              partitions = 1;
              partitionType = "count";
            }
          );
        };

        packages =
          {
            default = fuuka-bot;
          }
          // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
            fuuka-bot-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (
              commonArgs
              // {
                inherit cargoArtifacts;
              }
            );
          };

        apps.default = flake-utils.lib.mkApp {
          drv = fuuka-bot;
        };

        devShells = {
          default = craneLib.devShell {
            # Inherit inputs from checks.
            checks = self.checks.${system};

            # Extra inputs can be added here; cargo and rustc are provided by default.
            packages = [
              pkgs.rust-analyzer
              pkgs.rustfmt
              pkgs.clippy
            ];
          };
          fhs =
            (pkgs.buildFHSUserEnv {
              name = "fuuka-bot-fhs-devshell";
              targetPkgs = pkgs: [
                pkgs.gcc
                pkgs.rust-analyzer
                pkgs.rustfmt
                pkgs.clippy
                pkgs.pkg-config
                # Add additional build inputs here
                pkgs.openssl
                pkgs.sqlite
                fenix.packages.${system}.stable.toolchain
              ];
              extraOutputsToInstall = [ "dev" ];
            }).env;
        };
      }
    );
}
