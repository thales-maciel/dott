{
  inputs = {
    flake-parts = {
      inputs.nixpkgs-lib.follows = "nixpkgs";
      url = "github:hercules-ci/flake-parts";
    };

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "x86_64-darwin" ];

      perSystem = { pkgs, lib, config, ...}:
        let
          src = inputs.self;
          inherit (lib.importTOML (src + "/Cargo.toml")) package;
        in
        {
          packages = {
            default = pkgs.rustPlatform.buildRustPackage {
              pname = package.name;
              inherit (package) version;
              inherit src;
              cargoLock.lockFile = (src + "/Cargo.lock");
            };
          };

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
              cargo-deny
            ];
          };

          apps = {
            default = {
              program = "${config.packages.default}/bin/dotr";
              type = "app";
            };
          };
        };
    };
}
