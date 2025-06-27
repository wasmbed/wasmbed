{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
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

    crates = directories ./crates;
    src = craneLib.cleanCargoSource ./.;
    workspaceDeps = craneLib.buildDepsOnly {
      inherit src;
      strictDeps = true;
      doCheck = false;
    };

    # makeAttrs :: list -> attrset
    # Takes a list and a function that produces { name, value } for each element.
    makeAttrs = list: fn: builtins.listToAttrs (map fn list);

    # directories :: str -> [str]
    directories = path:
      builtins.attrNames
        (lib.attrsets.filterAttrs
          (_: type: type == "directory")
          (builtins.readDir path));

    # buildCrate :: str -> drv
    buildCrate = name: craneLib.buildPackage {
      inherit name src;
      strictDeps = true;
      doCheck = false;
      cargoArtifacts = workspaceDeps;
      cargoExtraArgs = "-p ${name}";
      meta.mainProgram = name;
    };

    # testCrate :: str -> drv
    testCrate = name: craneLib.cargoTest {
      name = "${name}-test";
      inherit src;
      strictDeps = true;
      cargoArtifacts = workspaceDeps;
      cargoExtraArgs = "-p ${name}";
    };

    # clippyCrate :: str -> drv
    clippyCrate = name: craneLib.cargoClippy {
      name = "${name}-clippy";
      inherit src;
      strictDeps = true;
      cargoArtifacts = workspaceDeps;
      cargoClippyExtraArgs = "-p ${name} -- --deny warnings";
    };

    # fmtCrate :: str -> drv
    fmtCrate = name: craneLib.cargoFmt {
      name = "${name}-fmt";
      inherit src;
      structDeps = true;
      cargoArtifacts = workspaceDeps;
      cargoFmtExtraArgs = "-p ${name}";
    };
  in {
    checks =
      makeAttrs
        crates
        (name: { name = "fmt-${name}"; value = fmtCrate name; })
      //
      makeAttrs
        crates
        (name: { name = "clippy-${name}"; value = clippyCrate name; })
      //
      makeAttrs
        crates
        (name: { name = "test-${name}"; value = testCrate name; })
    ;

    packages =
      lib.attrsets.genAttrs
        crates
        (name: buildCrate name)
      //
      {
        wasmbed-diagrams = pkgs.stdenvNoCC.mkDerivation {
          name = "wasmbed-diagrams";
          src = ./resources/diagrams;
          nativeBuildInputs = with pkgs; [ gnumake plantuml ];
          buildPhase = ''
            make svg
          '';
          installPhase = ''
            mkdir -p $out
            cp *.svg $out
          '';
        };
        defmt-print = craneLib.buildPackage {
          pname = "defmt-print";
          version = "1.0.0";
          src = pkgs.fetchCrate {
            pname = "defmt-print";
            version = "1.0.0";
            sha256 = "sha256-rio5kAL6NR7vBtjPF0GxDcINeWw+LuZWe7nFN0UkdBg=";
          };
          doCheck = false;
        };
      };

    devShells.default = craneLib.devShell {
      packages = [
        pkgs.gnumake
        pkgs.k3d
        pkgs.kubectl
        pkgs.plantuml
        pkgs.qemu
        pkgs.socat
        self.packages.${system}.defmt-print
      ];
    };

    dockerImages.wasmbed-gateway = pkgs.dockerTools.buildLayeredImage {
      name = "wasmbed-gateway";
      config = {
        Cmd = [
          (lib.meta.getExe self.packages.${system}.wasmbed-gateway)
        ];
      };
    };
  });
}
