# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0

import typer
import glob
import util
from plumbum import colors, FG
from plumbum.cmd import (
    rg,
    nix,
    alejandra,
    prettier,
    glow,
    taplo,
    cargo,
    ruff,
    addlicense,
)

root_path = util.root_path()
app = typer.Typer()


@app.command()
def todo():
    for file in glob.iglob("./**/TODO.md", root_dir=root_path, recursive=True):
        print(colors.blue & colors.bold & colors.underline | f">>> FILE: {file} <<<")
        glow[file] & FG
        print()

    rg[
        "TODO",
        root_path,
        "--iglob",
        "!TODO.md",
        "--iglob",
        "!/projects/toolkit/todo.bash",
    ] & FG


@app.command()
def check():
    print("Checking Nix files")
    nix["flake", "check", root_path, "--all-systems"] & FG

    print("Checking Nix files formatting")
    alejandra[root_path, "--check"] & FG

    print("Checking YAML, TS, JS, HTML, CSS, JSON and Markdown files")
    # Prettier does weird things with the CWD even when setting an absolute path
    prettier[".", "--check"].with_cwd(root_path) & FG

    print("Checking TOML files formatting")
    taplo["lint"].with_cwd(root_path) & FG

    print("Checking TOML files formatting")
    taplo["format", "--check"].with_cwd(root_path) & FG

    print("Checking Rust files")
    cargo["clippy"].with_cwd(root_path) & FG
    cargo["check-all-features"].with_cwd(root_path) & FG

    print("Checking Rust files formatting")
    cargo["fmt", "--check"].with_cwd(root_path) & FG

    print("Checking Python files")
    ruff["check", root_path] & FG

    print("Checkign Python files formatting")
    ruff["format", "--check", root_path] & FG

    print("Checking license headers")
    addlicense[
        "-s",
        "-l",
        "mpl",
        "-ignore",
        "**/node_modules/**",
        "-ignore",
        "**/target/**",
        "-check",
        root_path,
    ] & FG

    print()
    print(colors.green & colors.underline | ">>> Everything is ok! <<<")


@app.command()
def fix():
    print("Fixing Nix files formatting")
    alejandra[root_path] & FG

    print("Fixing YAML, TS, JS, HTML, CSS, JSON and Markdown files")
    # Prettier does weird things with the CWD even when setting an absolute path
    prettier[root_path, "--write"].with_cwd(root_path) & FG

    print("Checking TOML files")
    taplo["format"].with_cwd(root_path) & FG

    print("Fixing Rust files sematics")
    cargo["clippy", "--fix", "--allow-dirty"].with_cwd(root_path) & FG

    print("Fixing Rust files formatting")
    cargo["fmt"].with_cwd(root_path) & FG

    print("Fixing Python files")
    ruff["check", root_path, "--fix"] & FG

    print("Fixing Python files formatting")
    ruff["format", root_path] & FG

    print("Fixing license headers")
    addlicense[
        "-s",
        "-l",
        "mpl",
        "-ignore",
        "**/node_modules/**",
        "-ignore",
        "**/target/**",
        root_path,
    ] & FG

    print()
    print(colors.green & colors.underline | ">>> Done! <<<")


def main():
    app()


if __name__ == "__main__":
    main()
