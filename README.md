# ZEL.ZIP Monorepo

> [!IMPORTANT]  
> This document describes the development workflow of this repository, if you are searching for user-facing documentation please refer to the [ZEL.ZIP documentation](https://docs.zel.zip).
>
> If you don't know what is ZEL.ZIP, feel free to visit [our main page](https://zel.zip).

## Getting Started

This repository holds most of the code of the ZEL.ZIP project, its development workflow is based on the use of [Nix flake devShells](https://nixos.wiki/wiki/Development_environment_with_nix-shell#nix_develop) and a custom build terminal toolkit called `dev`.

Enter the devShell:

```sh
$ nix develop
```

Setup the monorepo:
```sh
$ dev setup
```

If you are going to work with a WASM library (`icebrk-wasm`, etc) remember to compile and build the NPM packages first:

```sh
$ dev wasm
```

This CLI tool allows for the most common development actions:

```sh
# Checking if the code will pass the CI/CD checks
dev check

# Try to fix some code issues
dev fix

# Check missing TODO tasks
dev todo
```

## TODO comments format

TODO comments use a simple format that tries to convey a little more info about the issue at a first glance: `TODO(<KIND>): <MESSAGE>`, where `<KIND>` can be:

- `IMPROVE`: This code can be improved.
- `CLEANUP`: This code, without adding new features, can be made easier to understand.
- `IMPLEMENT`: Here is missing a new feature.
- `DISCOVER`: Related to reverse engineering, the following code interacts with some system or format not fully understood yet.
- `TRACK: <URL>`: This issue depends on a fix by a third-party, the `<URL>` points to the relevant status about the development of the fix, usually being a code issue in GitHub, GitLab, Forgejo, etc.

## `dirty/` directories

Any directory called `dirty/` will be ignore by lints and Git, feel free to add any required file for early testing.

## Ignore-like file generation

Ignore-like files (`.gitignore` and folks) are builded by merging all the `.ignore` files inside the `//ignores` directory, to regenerate these files run:

```sh
$ dev ignores
```

## Legal notice

This project is a fan-made homebrew creation developed independently and is not affiliated with, endorsed by, or associated with Nintendo Co., Ltd or any of its subsidiaries, affiliates, or partners. All trademarks and copyrights referenced are the property of their respective owners.

All files presented here are under the [Mozilla Public License Version 2.0](./LICENSE.txt) otherwise noted.
