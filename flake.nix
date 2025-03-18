{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane = {
      url = "github:ipetkov/crane";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
      inherit (pkgs) lib;

      rustToolchainFor = p: p.rust-bin.stable.latest.default.override {
        targets = [
          "wasm32-unknown-unknown"
          "x86_64-unknown-linux-gnu"
          "x86_64-pc-windows-gnu"
        ];
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchainFor;

      glfw = pkgs.glfw.overrideAttrs (cur: prev: {
        cmakeFlags = []; # by default, static linking is disabled here
        # for some reason, the default glfw package hardcodes a nix store path to libGL
        # see: https://github.com/NixOS/nixpkgs/pull/47175
        # this makes it impossible to run the binary on another system
        # I'd much rather just load whatever we find on LD_LIBRARY_PATH etc, since this is much easier to control
        env = {};
        postPatch = "true";
      });

      native = rec {
        nativeBuildInputs = [
          pkgs.pkg-config
        ];
        buildInputs = [
          pkgs.openssl.dev
          glfw
          pkgs.xorg.libX11 
          pkgs.xorg.libXcursor 
          pkgs.xorg.libXi 
          pkgs.xorg.libXrandr
          pkgs.xorg.libXinerama
          pkgs.libxkbcommon 
          pkgs.xorg.libxcb  
          pkgs.libglvnd
          pkgs.alsa-lib
        ];
        build = path: nm:
          let
            src = lib.cleanSourceWith {
              src = path;
              filter = path: type:
                (lib.hasSuffix "\.html" path) ||
                (lib.hasSuffix "\.js" path) ||
                (lib.hasSuffix "\.css" path) ||
                (lib.hasInfix "/assets/" path) ||
                (craneLib.filterCargoSources path type)
              ;
            };
            commonArgs = {
              inherit src nativeBuildInputs buildInputs;
              strictDeps = true;
              CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
              CARGO_BUILD_RUSTFLAGS="-L ${glfw}/lib";
              inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
            };
            cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
              doCheck = false;
            });
          in
            craneLib.buildPackage (commonArgs // {
              inherit cargoArtifacts;
              pname = nm;
              cargoExtraArgs = "-p ${nm}";
            });
      };

      wasm = rec {
        build = path: nm:
          let
            src = lib.cleanSourceWith {
              src = path;
              filter = path: type:
                (lib.hasSuffix "\.html" path) ||
                (lib.hasSuffix "\.js" path) ||
                (lib.hasSuffix "\.css" path) ||
                (lib.hasInfix "/assets/" path) ||
                (craneLib.filterCargoSources path type)
              ;
            };
            commonArgs = {
              inherit src;
              strictDeps = true;
              CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
              buildInputs = [];
              inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
              wasm-bindgen-cli = pkgs.buildWasmBindgenCli rec {
                src = pkgs.fetchCrate {
                  pname = "wasm-bindgen-cli";
                  version = "0.2.100";
                  hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
                };
                cargoDeps = pkgs.rustPlatform.fetchCargoVendor {
                  inherit src;
                  inherit (src) pname version;
                  hash = "sha256-qsO12332HSjWCVKtf1cUePWWb9IdYUmT+8OPj/XP2WE=";
                };
              };
            };
          in
            craneLib.buildTrunkPackage (commonArgs // rec {
              pname = nm;
              cargoExtraArgs = "-p ${nm}";
              cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
                inherit cargoExtraArgs;
                doCheck = false;
              });
              preBuild = ''
                cd ./crates/throwshade
              '';
              postBuild = ''
                mv ./dist ../..
                cd ../..
              '';
            });
      };

      shell = craneLib.devShell {
        packages = [
          pkgs.trunk
          pkgs.rust-analyzer
          pkgs.glxinfo
          pkgs.cmake
        ] ++ native.nativeBuildInputs ++ native.buildInputs;
        LIBRARY_PATH = "$LIBRARY_PATH:${pkgs.lib.makeLibraryPath native.buildInputs}";
        RUSTFLAGS="-L ${glfw}/lib";
        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath native.buildInputs}";
      };
    in {
      packages.${system}.glfw = glfw;
      inherit shell native wasm;
      devShells.${system}.default = shell;
    };
}
