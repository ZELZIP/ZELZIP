import typer
import glob
import util
from plumbum import colors, FG
from plumbum.cmd import rg, nix, alejandra, prettier, glow, taplo, cargo, ruff

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
    prettier[root_path, "--check"] & FG

    print("Checking TOML files")
    taplo["format", "--check"] & FG

    print("Checking Rust files")
    cargo["clippy"] & FG
    cargo["check-all-features"] & FG

    print("Checking Rust files formatting")
    cargo["fmt", "--check"] & FG

    print("Checking Python files")
    ruff["check", root_path] & FG

    print("Checkign Python files formatting")
    ruff["format", "--check"] & FG

    print()
    print(colors.green & colors.underline | ">>> Everything is ok! <<<")


@app.command()
def fix():
    print("Fixing Nix files formatting")
    alejandra[root_path] & FG

    print("Fixing YAML, TS, JS, HTML, CSS, JSON and Markdown files")
    prettier[root_path, "--write"] & FG

    print("Checking TOML files")
    taplo["format"] & FG

    print("Fixing Rust files sematics")
    cargo["clippy", "--fix", "--allow-dirty"] & FG

    print("Fixing Rust files formatting")
    cargo["fmt"] & FG

    print("Fixing Python files")
    ruff["check", root_path, "--fix"] & FG

    print("Fixing Python files formatting")
    ruff["format"] & FG

    print()
    print(colors.green & colors.underline | ">>> Done! <<<")


def main():
    app()


if __name__ == "__main__":
    main()
