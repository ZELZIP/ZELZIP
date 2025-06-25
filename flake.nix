{
  description = "GOTRE's monorepo for packages and services";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";

    toolkit.url = "./projects/toolkit";
    toolkit.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = [
        "x86_64-linux"
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: {
        # Modify `pkgs` to add overlays
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.fenix.overlays.default
          ];
          config = {};
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            # Language toolchains, runtimes, LSPs, linters and formatters
            ## Nixlang
            nixd
            alejandra

            ## Markdown
            marksman
            glow

            ## Bash
            bash-language-server
            shellcheck
            shfmt

            ## TOML
            taplo

            ## Rust
            fenix.complete.toolchain
            cargo-all-features

            ## YAML, TS, JS, HTML, CSS, JSON, Markdown
            pnpm
            nodejs
            nodePackages.prettier

            ## Python
            ruff

            # Apps
            ## Grepping inside multiple files
            ripgrep

            ## The toolkit of the project
            (with pkgs.python3Packages;
              buildPythonPackage {
                pname = "zelzip-toolchain";
                version = "0.0.0";

                src = ./projects/toolkit;

                buildInputs = [setuptools];

                propagatedBuildInputs = [typer plumbum];

                pyproject = true;
              })
          ];
        };
      };
    });
}
