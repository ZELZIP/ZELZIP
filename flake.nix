{
  description = "GOTRE's monorepo for packages and services";

  inputs = {
    # TODO(TRACK: https://github.com/NixOS/nix/issues/10683):
    #   Use `shallow=1` and `git+https` to avoid insane slow download times
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs.follows = "nixpkgs";

    mozilla.url = "github:mozilla/nixpkgs-mozilla/master";
    mozilla.inputs.nixpkgs.follows = "nixpkgs";

    toolkit.url = ./projects/toolkit;
    toolkit.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: {
      systems = [
        "x86_64-linux"
      ];

      perSystem = {
        pkgs,
        inputs',
        system,
        ...
      }: {
	# Modify `pkgs` to add overlays
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.mozilla.overlays.rust
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

            ## Bash
            bash-language-server
            shellcheck
            shfmt

            ## TOML
            taplo

	    ## Rust
	    (pkgs.rustChannelOf { rustToolchain = ./rust-toolchain.toml; }).rust

            ## Shared (YAML, TS, JS, HTML, CSS, JSON, Markdown)
            nodePackages.prettier

            # Apps
            ## Wii emulator
            dolphin-emu

            ## Logging and prompts for shell scripts
            gum

            ## Grepping inside multiple files
            ripgrep

            inputs'.toolkit.packages.default
          ];
        };
      };
    });
}
