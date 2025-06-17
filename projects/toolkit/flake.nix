{
  description = "Package definitions for scripts toolkit";

  inputs = {
    nixpkgs.url = "git+https://github.com/NixOS/nixpkgs?shallow=1&ref=nixos-24.11";

    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs @ {flake-parts, ...}:
    flake-parts.lib.mkFlake {inherit inputs;} ({...}: let
      makeScript = {
        pkgs,
        name,
      }:
        pkgs.writeShellScriptBin
        "_${name}"
        (
          builtins.readFile ./_setup.bash
          + pkgs.lib.concatStrings (
            map
            (fileName: builtins.readFile (./_modules + "/${fileName}"))
            (builtins.attrNames (builtins.readDir ./_modules))
          )
          + builtins.readFile (./. + "/${name}.bash")
        );
    in {
      systems = [
        "x86_64-linux"
        "aarch64-darwin"
      ];

      perSystem = {pkgs, ...}: {
        packages.default = pkgs.symlinkJoin {
          name = "mergedToolkitScrips";
          paths = map (name: (makeScript {
            inherit pkgs;
            inherit name;
          })) ["todo" "fix" "check" "test" "run"];
        };
      };
    });
}
