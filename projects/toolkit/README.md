# Toolkit

> A set of tools to ease the usage of the monorepo.

## Usage

Add the [local flake](./flake.nix) to the inputs of your flake and use the `default` package as desired.

### Example

```nix
{
  inputs {
    nixpkgs.url = "github:NixOS/nixpkgs";

    toolkit.url = "<Wherever the Nix Flake is located>";
    toolkit.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, toolkit, ... }:
    let
      system = "x86_64-linux"; # Probably you want to use a "system" handler like flake-parts instead
    in {
      devShells.${system}.default = nixpkgs.legacyPackages.${system}.mkShell {
        packages = [
          toolkit.packages.${system}.default
        ];
      };
    };
}
```

## Design

This set of scripts is wrapped around a single Nix package inside the [local flake](./flake.nix), all of them are written using [GNU Bash](https://www.gnu.org/software/bash/).

They expect and use a little environment set by the [`_setup.bash`](./_setup.bash) script, this initialization file is prepended at the start of each of them by Nix, using a shebang is then only a hint for IDEs and linters.

Also any file inside [`./_modules`] will be blindly injected between the setup script and the script content.
