{
  description = "koi development environment";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustfmt
            ];
            shellHook = ''
              echo "cargo"
              which cargo
              cargo --version
            '';
          };
        }
      );

      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "koi";
            version = "1.0.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
        }
      );

      apps = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
          };
        in
        {
          default = {
            type = "app";
            buildInputs = with pkgs; [ cargo ];
            program = toString (
              pkgs.writeShellScript "cargo-run" ''
                cargo run -- "$@"
              ''
            );
          };
          cargo = {
            type = "app";
            buildInputs = with pkgs; [ cargo ];
            program = toString (
              pkgs.writeShellScript "cargo-general" ''
                cargo "$@"
              ''
            );
          };
          test = {
            type = "app";
            buildInputs = with pkgs; [ cargo ];
            program = toString (
              pkgs.writeShellScript "cargo-test" ''
                cargo test -- "$@"
              ''
            );
          };
          clippy = {
            type = "app";
            buildInputs = with pkgs; [
              cargo
              cargo-clippy
            ];
            program = toString (
              pkgs.writeShellScript "cargo-clippy" ''
                cargo clippy
              ''
            );
          };
          fix = {
            type = "app";
            buildInputs = with pkgs; [
              cargo
              cargo-clippy
            ];
            program = toString (
              pkgs.writeShellScript "cargo-fix" ''
                cargo clippy --allow-dirty --fix --bin "koi"
              ''
            );
          };
          fmt = {
            type = "app";
            buildInputs = with pkgs; [ rustfmt ];
            program = toString (
              pkgs.writeShellScript "rustfmt-all" ''
                rustfmt src/**/*.rs
              ''
            );
          };
        }
      );
    };
}
