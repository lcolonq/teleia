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
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        inherit (pkgs) lib;

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
        craneLib = ((crane.mkLib pkgs).overrideToolchain rustToolchain).overrideScope (_final: _prev: {
          inherit (import nixpkgs-for-wasm-bindgen { inherit system; }) wasm-bindgen-cli;
        });

        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasSuffix "\.html" path) ||
            (lib.hasSuffix "\.scss" path) ||
            (lib.hasInfix "/assets/" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          CARGO_BUILD_TARGET = "wasm32-unknown-unknown";
          buildInputs = [
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          doCheck = false;
        });

        teleia = craneLib.buildTrunkPackage (commonArgs // {
          inherit cargoArtifacts;
          wasm-bindgen-cli = pkgs.wasm-bindgen-cli.override {
            version = "0.2.90";
            hash = "sha256-X8+DVX7dmKh7BgXqP7Fp0smhup5OO8eWEhn26ODYbkQ=";
            cargoHash = "sha256-ckJxAR20GuVGstzXzIj1M0WBFj5eJjrO2/DRMUK5dwM=";
          };
        });
      in {
        packages.default = teleia;

        devShells.default = craneLib.devShell {
          packages = [
            pkgs.trunk
            pkgs.rust-analyzer
            pkgs.glxinfo
            pkgs.alsa-lib
            pkgs.cmake
            pkgs.pkg-config
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
          ];
          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
            pkgs.lib.makeLibraryPath [
              pkgs.xorg.libX11 
              pkgs.xorg.libXcursor 
              pkgs.xorg.libXi 
              pkgs.libxkbcommon 
              pkgs.xorg.libxcb  
              pkgs.libglvnd
            ]
          }";
        };
      });
}
