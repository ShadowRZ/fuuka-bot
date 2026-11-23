{
  description = "Fuuka Bot";

  inputs = {
    nixpkgs = {
      url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    };

    crane = {
      url = "github:ipetkov/crane";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      crane,
      rust-overlay,
      ...
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
    in
    {
      checks = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          packages = pkgs.callPackage ./nix/packages.nix {
            inherit crane rust-overlay;
          };
        in
        {
          inherit (packages)
            fuuka-bot
            fuuka-bot-clippy
            fuuka-bot-doc
            fuuka-bot-fmt
            fuuka-bot-nextest
            ;
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          packages = pkgs.callPackage ./nix/packages.nix {
            inherit crane rust-overlay;
          };
        in
        rec {
          inherit (packages) fuuka-bot;
          default = fuuka-bot;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          devshell = pkgs.callPackage ./nix/devshell.nix {
            inherit rust-overlay;
          };
        in
        {
          default = devshell;
        }
      );
    };
}
