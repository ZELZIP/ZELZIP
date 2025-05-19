#!/usr/bin/env bash

gotre::log::info "Setting script files permissions"
chmod 644 ./projects/toolkit/**/*.bash

gotre::log::info "Fixing errors in Bash script files"

# The command fails if there are unfixable errors and we
# don't care as it'll be caught by the '//projects/toolkit/check.bash' script
set +e
shellcheck -xf diff ./**/*.bash | patch -p1
set -e

gotre::log::info "Formatting Bash script files"
shfmt --write ./**/*.bash

gotre::log::info "Formatting Nix files formatting"
alejandra .

gotre::log::info "Fixing YAML, TS, JS, HTML, CSS, JSON and Markdown"
prettier . --write

gotre::log::info "Done!"
