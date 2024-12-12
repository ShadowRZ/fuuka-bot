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

              toolchain = (
                fenix.stable.withComponents [
                  "cargo"
                  "clippy"
                  "rust-src"
                  "rustc"
                  "rustfmt"
                  "llvm-tools"
                ]
              );

              craneLib = (crane.mkLib pkgs).overrideToolchain (_: toolchain);

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
                  }
                );
              };

              packages =
                {
                  default = fuuka-bot;
                }
                // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
                  fuuka-bot-llvm-coverage = craneLib.cargoLlvmCov (
                    commonArgs
                    // {
                      inherit cargoArtifacts;
                    }
                  );
                };

              devShells = {
                default = craneLib.devShell {
                  # Inherit inputs from checks.
                  inherit (self') checks;

                  # Extra inputs can be added here; cargo and rustc are provided by default.
                  packages = [
                    toolchain.rustfmt
                    toolchain.clippy
                    toolchain.llvm-tools
                  ];

                  RUST_SRC_PATH = "${fenix.stable.rust-src}/lib/rustlib/src/rust/library";
                };
              };
            }
          );
      }
    );
}
