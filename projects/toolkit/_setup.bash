#!/usr/bin/env bash

# Setup an "strict Bash" environment

set -euo pipefail
shopt -s globstar failglob

cd "$(git rev-parse --show-toplevel)" || exit
