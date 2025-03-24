{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
    crane,
  }:
  flake-utils.lib.eachDefaultSystem (system: let
    inherit (nixpkgs) lib;

    overlays = [ (import rust-overlay) ];
    pkgs = import nixpkgs { inherit system overlays; };

    mkToolchain = p: p.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    craneLib = (crane.mkLib pkgs).overrideToolchain mkToolchain;
    src = craneLib.cleanCargoSource ./.;

    listDirectories = path:
      builtins.attrNames
        (lib.attrsets.filterAttrs
          (_: type: type == "directory")
          (builtins.readDir path));
    packages = listDirectories ./crates;

    buildPackage = { name, doCheck }: craneLib.buildPackage {
      inherit src doCheck;
      inherit
        (craneLib.crateNameFromCargoToml { cargoToml = ./crates/${name}/Cargo.toml; })
        pname;
      cargoExtraArgs = "-p ${name}";
    };
  in {
    checks =
      lib.attrsets.genAttrs
        packages
        (name: buildPackage { inherit name; doCheck = true; });

    packages =
      lib.attrsets.genAttrs
        packages
        (name: buildPackage { inherit name; doCheck = false; });

    devShells.default = craneLib.devShell {};

    dockerImages.wasmbed-operator = pkgs.dockerTools.buildLayeredImage {
      name = "wasmbed-operator";
      config = {
        Cmd = [
          "${self.packages.${system}.wasmbed-operator}/bin/wasmbed-operator"
          "controller"
        ];
      };
    };
  });
}
