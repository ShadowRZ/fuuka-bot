update:
  cargo update

update-flake:
  nix --extra-experimental-features 'nix-command flakes' flake update --verbose -L --commit-lock-file

format:
  cargo fmt

clippy:
  cargo clippy

build:
  cargo build -r

build-aarch64-musl:
  nix --extra-experimental-features 'nix-command flakes' develop '.#aarch64-unknown-linux-musl' --command cargo build -r --target aarch64-unknown-linux-musl -F bundled-sqlite
