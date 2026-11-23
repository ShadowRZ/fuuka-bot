{
  pkgs,
  lib,
  pkg-config,
  openssl,
  sqlite,
  # From flake input
  rust-overlay,
}:

let
  rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
  toolchain = rust-bin.fromRustupToolchainFile ../rust-toolchain.toml;
in
pkgs.mkShell {
  packages = [
    (toolchain.override {
      extensions = [
        "rust-analyzer"
        "rust-src"
      ];
    })
  ];

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    openssl
    sqlite
  ];
}
