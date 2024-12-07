{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.x86_64-linux.default = pkgs.mkShell {
        buildInputs = [
          pkgs.pkg-config
          pkgs.llvm
          pkgs.clang
          pkgs.llvmPackages.libclang
          pkgs.openssl
        ];
        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
          with pkgs;
          pkgs.lib.makeLibraryPath [
            libGL 
            xorg.libX11 
            xorg.libXcursor 
            xorg.libXi 
            libxkbcommon 
            xorg.libxcb  
          ]
        }";
      };
    };
}
