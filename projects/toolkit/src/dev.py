# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0

import typer
import glob
from typing_extensions import Annotated
from util import get_root_path
from plumbum import colors, FG, local
from plumbum.cmd import (
    rg,
    nix,
    alejandra,
    pnpm,
    glow,
    taplo,
    cargo,
    ruff,
    addlicense,
    jq,
    docker,
)

wasm_pack = local["wasm-pack"]
wasm_opt = local["wasm-opt"]

root_path = get_root_path()
app = typer.Typer(help="ZELZIP internal monorepo toolkit")


WASM_PROJECTS = ["icebrk"]


@app.command()
def build_docker_static_server(project_name: str):
    """
    Build a Docker image for static server
    """

    pnpm[
        "run",
        "--dir",
        f"{root_path}/projects/{project_name}",
        "build",
        "--outDir",
        f"{root_path}/dockerfiles/static_server/dist",
    ] & FG
    docker[
        "build",
        "-t",
        f"ghcr.io/zelzip/{project_name}:latest",
        "dockerfiles/static_server/",
    ] & FG


@app.command()
def build_docker_typedoc(project_name: str):
    """
    Build a Docker image for Typedoc static server
    """

    pnpm[
        "typedoc",
        "--tsconfig",
        f"{root_path}/projects/{project_name}/tsconfig.json",
        "--out",
        f"{root_path}/dockerfiles/static_server/dist",
    ] & FG
    docker[
        "build",
        "-t",
        f"ghcr.io/zelzip/{project_name}_typedoc:latest",
        "dockerfiles/static_server/",
    ] & FG


@app.command()
def setup_pnpm():
    """
    Setup the monorepo PNPM dependencies.
    """

    pnpm["install"] & FG


@app.command()
def wasm(
    projects: Annotated[
        list[str] | None,
        typer.Argument(
            help="Optional set of space separated projects that their WASM variant should be compiled, if not specified defaults to compile all of them."
        ),
    ] = None,
):
    """
    Build WASM projects.
    """

    projects_names = []

    if projects is None:
        projects_names = WASM_PROJECTS
    else:
        projects_names = projects

    for project in projects_names:
        out_path = f"{root_path}/projects/{project}_wasm"

        wasm_pack[
            "build", f"{root_path}/projects/{project}/", "--out-dir", out_path
        ] & FG

        wasm_filename = glob.glob("*.wasm", root_dir=out_path)[0]
        wasm_file_path = f"{out_path}/{wasm_filename}"

        wasm_opt[
            "-Oz", "--enable-bulk-memory-opt", "-o", wasm_file_path, wasm_file_path
        ] & FG

        package_json_path = f"{out_path}/package.json"

        new_package_json_text = jq[f'.name = "@zelzip/{project}"', package_json_path]()

        with open(package_json_path, "w") as file:
            file.write(new_package_json_text)


@app.command()
def todo():
    """
    List all inline and in-file TODO tasks.
    """

    for file in glob.iglob("./**/TODO.md", root_dir=root_path, recursive=True):
        print(colors.blue & colors.bold & colors.underline | f">>> FILE: {file} <<<")
        glow[f"{root_path}/{file}"] & FG
        print()

    rg[
        "TODO",
        root_path,
        "--iglob",
        "!TODO.md",
        "--iglob",
        "!/projects/toolkit/src/dev.py",
        "--iglob",
        "!/README.md",
        "--hidden",
    ] & FG


@app.command()
def check():
    """
    Check the integrity of the code and the repository.
    """

    print("Checking Nix files")
    nix["flake", "check", root_path, "--all-systems"] & FG

    print("Checking Nix files formatting")
    alejandra[root_path, "--check"] & FG

    print("Checking web related files")
    pnpm["eslint", root_path] & FG
    # Prettier does weird things with the CWD even when setting an absolute path
    pnpm["prettier", ".", "--check"].with_cwd(root_path) & FG

    print("Checking TOML files formatting")
    taplo["lint"].with_cwd(root_path) & FG

    print("Checking TOML files formatting")
    taplo["format", "--check"].with_cwd(root_path) & FG

    print("Checking Rust files")
    cargo["clippy", "--", "-D", "warnings"].with_cwd(root_path) & FG

    with local.env(RUSTFLAGS="-D warnings"):
        cargo["check-all-features", "--all-targets"].with_cwd(root_path) & FG

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
        "-ignore",
        "**/*_wasm/**",
        "-ignore",
        "**/*_typedoc/**",
        "-check",
        root_path,
    ] & FG

    print()
    print(colors.green & colors.underline | ">>> Everything is ok! <<<")


@app.command()
def fix():
    """
    Try to fix some code issues.
    """

    print("Fixing Nix files formatting")
    alejandra[root_path] & FG

    print("Fixing web related files")
    pnpm["eslint", root_path, "--fix"] & FG
    # Prettier does weird things with the CWD even when setting an absolute path
    pnpm["prettier", root_path, "--write"].with_cwd(root_path) & FG

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


@app.command()
def ignores():
    """
    Build new ignore-like files.
    """

    print("Building ignore-like files")

    ignore_lines = [
        "# @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@",
        "# @  AUTOGENERATED FILE WITH THE ZELZIP TOOLKIT, DO NOT EDIT!  @",
        "# @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@",
        "",
    ]

    print("Reading .ignore files")
    for path in glob.iglob(
        f"{root_path}/ignores/*.ignore", root_dir=root_path, recursive=True
    ):
        print(f"  {path}")

        with open(f"{path}", "r") as file:
            ignore_lines.append(f">>> FILE: {path} <<<")
            ignore_lines += file.readlines()

    with open(f"{root_path}/.gitignore", "w") as file:
        for line in ignore_lines:
            print(line)
            file.write(line + "\n")

        file.truncate()


@app.command()
def test():
    """
    Run all test suites.
    """

    cargo["test"].with_cwd(root_path) & FG


def main():
    app()


if __name__ == "__main__":
    main()
