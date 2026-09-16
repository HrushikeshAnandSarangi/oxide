{
  description = "Minimal Rust server, built fully static via musl, for measuring Oxide's Nix-vs-naive-Docker container size claim";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.${system}.default = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
        pname = "oxide-minimal-app";
        version = "0.1.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
    };
}
