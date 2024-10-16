{
  description = "koi development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    git-hooks = {
      url = "github:cachix/git-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.nixpkgs-stable.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      git-hooks,
    }:
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
      checks = forAllSystems (system: {
        pre-commit-check = git-hooks.lib.${system}.run {
          src = ./.;
          hooks = {
            # format nix
            nixfmt-rfc-style.enable = true;
            # format markdown
            denofmt.enable = true;
            # format rust
            rustfmt.enable = true;
            # check github actions yml
            # actionlint.enable = true;
          };
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          pre-commit-check = self.checks.${system}.pre-commit-check;
        in
        {
          default = pkgs.mkShell {
            buildInputs = pre-commit-check.enabledPackages ++ [
              pkgs.cargo
              pkgs.rustfmt
            ];
            shellHook = ''
              echo "cargo"
              which cargo
              cargo --version
              ${pre-commit-check.shellHook}
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
