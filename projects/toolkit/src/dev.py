# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.
#
# SPDX-License-Identifier: MPL-2.0

import typer
import glob
import util
from plumbum import colors, FG, local
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
    jq,
)

wasm_pack = local["wasm-pack"]
wasm_opt = local["wasm-opt"]

root_path = util.root_path()
app = typer.Typer()


WASM_PROJECTS = ["icebrk"]


@app.command()
def wasm(project: list[str] = None):
    projects = []

    if project is None:
        projects = WASM_PROJECTS
    else:
        projects = project

    for project in projects:
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

        new_package_json_text = jq[f'.name = "@zel.zip/{project}"', package_json_path]()

        with open(package_json_path, "w") as file:
            file.write(new_package_json_text)


@app.command()
def todo():
    """
    List all inline and in-file TODO tasks.
    """

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
    """
    Check the integrity of the code and the repository.
    """

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


@app.command()
def ignores():
    """
    Build new ignore-like files.
    """

    print("Building ignore-like files")

    ignore_lines = [
        "# @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@",
        "# @  AUTOGENERATED FILE WITH THE ZEL.ZIP TOOLKIT, DO NOT EDIT!  @",
        "# @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@",
        "",
    ]

    print("Reading .ignore files")
    for path in glob.iglob("./ignores/*.ignore", root_dir=root_path, recursive=True):
        print(f"  {path}")

        with open(path, "r") as file:
            ignore_lines.append(f">>> FILE: {path} <<<")
            ignore_lines += file.readlines()

    for filename in [".gitignore", ".prettierignore"]:
        print(f"Writing {filename}")

        with open(f"{root_path}/{filename}", "w") as file:
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
