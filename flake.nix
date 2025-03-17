{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixpkgs-for-wasm-bindgen.url = "github:NixOS/nixpkgs/4e6868b1aa3766ab1de169922bb3826143941973";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, nixpkgs-for-wasm-bindgen, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      inherit (pkgs) lib;

      rustToolchain = pkgs.rust-bin.stable.latest.default.override {
        targets = [ "wasm32-unknown-unknown" "x86_64-unknown-linux-gnu" ];
      };
      craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (_final: _prev: {
        inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
      });
      src = lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          (lib.hasSuffix "\.html" path) ||
          (lib.hasSuffix "\.js" path) ||
          (lib.hasSuffix "\.css" path) ||
          (lib.hasInfix "/assets/" path) ||
          (craneLib.filterCargoSources path type)
        ;
      };

      native = rec {
        nativeBuildInputs = [
          pkgs.pkg-config
        ];
        buildInputs = [
          pkgs.openssl.dev
          pkgs.glfw
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
        commonArgs = {
          inherit src nativeBuildInputs buildInputs;
          strictDeps = true;
          CARGO_BUILD_TARGET = "x86_64-unknown-linux-gnu";
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
        };
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          doCheck = false;
        });
        build = nm: craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          pname = nm;
          cargoExtraArgs = "-p ${nm}";
        });
      };

      wasm = rec {
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
        build = nm: craneLib.buildTrunkPackage (commonArgs // rec {
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
        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath native.buildInputs}";
      };
    in {
      inherit shell native wasm;
      devShells.${system}.default = shell;
    };
}
