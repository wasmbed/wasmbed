{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }: let
    systems = with flake-utils.lib.system; [ x86_64-linux ];
  in flake-utils.lib.eachSystem systems (system: let
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
  in {
    devShells.default = pkgs.mkShell {
      packages = [ toolchain ];
    };
  });
}
