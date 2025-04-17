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
    workspaceDeps = craneLib.buildDepsOnly { inherit src; strictDeps = true; };

    crates = directories ./crates;

    # makeAttrs :: list -> attrset
    # Takes a list and a function that produces { name, value } for each element.
    makeAttrs = list: fn: builtins.listToAttrs (map fn list);

    # directories :: str -> [str]
    directories = path:
      builtins.attrNames
        (lib.attrsets.filterAttrs
          (_: type: type == "directory")
          (builtins.readDir path));

    # crateMeta :: str -> { pname: str; version: str; }
    crateMeta = name: {
      inherit
        (craneLib.crateNameFromCargoToml { cargoToml = ./crates/${name}/Cargo.toml; })
        pname version;
    };

    # buildCrate :: { name: str; doCheck: bool; } -> drv
    buildCrate = { name, doCheck }: craneLib.buildPackage {
      inherit (crateMeta name);
      inherit src doCheck;
      strictDeps = true;
      cargoArtifacts = workspaceDeps;
      cargoExtraArgs = "-p ${name}";
    };

    # clippyCrate :: str -> drv
    clippyCrate = name: craneLib.cargoClippy {
      inherit src;
      strictDeps = true;
      cargoArtifacts = workspaceDeps;
      cargoClippyExtraArgs = "-p ${name} -- --deny warnings";
    };
  in {
    checks =
      makeAttrs
        crates
        (name: {
          name = "test-${name}";
          value = buildCrate { inherit name; doCheck = true; };
        })
      //
      makeAttrs
        crates
        (name: {
          name = "clippy-${name}";
          value = clippyCrate name;
        })
    ;

    packages =
      lib.attrsets.genAttrs
        crates
        (name: buildCrate { inherit name; doCheck = false; });

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
