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
  }:
  flake-utils.lib.eachDefaultSystem (system: let
    inherit (nixpkgs) lib;
    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };
    toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    rustPlatform = pkgs.makeRustPlatform {
      cargo = toolchain;
      rustc = toolchain;
    };
    listDirectories = path:
      builtins.attrNames
        (lib.attrsets.filterAttrs
          (_: type: type == "directory")
          (builtins.readDir path));
    buildPackage = { name, doCheck }: rustPlatform.buildRustPackage {
      inherit name doCheck;
      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;
      auditable = false;
    };
    packages =
      map
        (dir: "wasmbed-${dir}")
        (listDirectories ./crates);
  in {
    checks =
      lib.attrsets.genAttrs
        packages
        (name: buildPackage { inherit name; doCheck = true; });
    packages =
      lib.attrsets.genAttrs
        packages
        (name: buildPackage { inherit name; doCheck = false; });
    devShells.default = pkgs.mkShell {
      packages = [ toolchain ];
    };
  });
}
