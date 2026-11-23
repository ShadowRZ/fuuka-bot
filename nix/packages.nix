{
  pkgs,
  lib,
  pkg-config,
  openssl,
  sqlite,
  # From flake input
  crane,
  rust-overlay,
}:

let
  rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
  toolchain = rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
  craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      # Default files from crane (Rust and cargo files)
      (craneLib.fileset.commonCargoSources ../.)
      # Also keep any markdown files
      (lib.fileset.fileFilter (file: file.hasExt "jinja") ../.)
    ];
  };

  # Common arguments can be set here to avoid repeating them later
  commonArgs = {
    inherit src;
    strictDeps = true;

    nativeBuildInputs = [
      pkg-config
    ];

    buildInputs = [
      # Add additional build inputs here
      openssl
      sqlite
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

  # Run tests with cargo-nextest
  fuuka-bot-nextest = craneLib.cargoNextest (
    commonArgs
    // {
      inherit cargoArtifacts;
      partitions = 1;
      partitionType = "count";
      cargoNextestPartitionsExtraArgs = "--no-tests=pass";
    }
  );
}
