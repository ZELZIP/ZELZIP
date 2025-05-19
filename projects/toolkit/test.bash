#!/usr/bin/env bash

cd "$GOTRE__SETUP__ORIGIN_PATH" || exit
_GOTRE__TEST__PROJECT_MANIFEST_PATH=$(realpath ./project.manifest)

if [ ! -f "$_GOTRE__TEST__PROJECT_MANIFEST_PATH" ]; then
  gotre::log::error "No project manifest file ('project.manifest') found!"
  gotre::log::error "Either you are not inside a project or this project doesn't use any special hooks"

  exit 1
fi

function gotre::test::_is_flag_enabled {
  grep -q "$1" "$_GOTRE__TEST__PROJECT_MANIFEST_PATH"
}

gotre::log::info "Running all tests in the project"

if gotre::test::_is_flag_enabled "rust"; then
  cargo test
fi
