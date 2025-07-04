# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";

    flake-parts.url = "github:hercules-ci/flake-parts";

    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
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
            (with fenix;
              combine [
                stable.toolchain
                targets.wasm32-unknown-unknown.stable.rust-std
              ])
            cargo-all-features
            wasm-pack
            binaryen

            ## YAML, TS, JS, HTML, CSS, JSON, Markdown
            pnpm
            nodejs
            nodePackages.prettier

            ## Python
            ruff

            # Apps
            ## Grepping inside multiple files
            ripgrep

            ## Add license headers to all compatible files
            addlicense

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
