#!/usr/bin/env bash

# Setup an "strict Bash" environment

set -euo pipefail
shopt -s globstar failglob

export GOTRE__SETUP__ORIGIN_PATH="$PWD"

cd "$(git rev-parse --show-toplevel)" || exit
