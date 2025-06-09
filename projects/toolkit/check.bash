#!/usr/bin/env bash

gotre::log::info "Checking script files permissions"
for script in ./projects/toolkit/**/*.bash; do
  if [ "$(stat -c "%a" "$script")" != "644" ]; then
    echo "The script '$script' doesn't have the correct permission"
    exit 1
  fi
done

gotre::log::info "Checking errors in Bash script files"
shellcheck ./**/*.bash

gotre::log::info "Checking formating in Bash script files"
shfmt --diff ./**/*.bash

gotre::log::info "Checking Nix files semantics"
nix flake check ./projects/toolkit --all-systems

gotre::log::info "Checking Nix files formatting"
alejandra . --check

gotre::log::info "Checking YAML, TS, JS, HTML, CSS, JSON and Markdown files"
prettier . --check

gotre::log::info "Checking TOML files"
taplo format --check

gotre::log::info "Checking Rust files semantics"
RUSTFLAGS="-D warnings" cargo clippy
RUSTFLAGS="-D warnings" cargo check-all-features

gotre::log::info "Checking Rust files formatting"
cargo fmt --check

gotre::log::info "Everything is right!"
