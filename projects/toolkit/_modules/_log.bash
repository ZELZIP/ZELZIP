#!/usr/bin/env bash

function gotre::log::_log {
  gum log --time rfc822 --level "$1" "$2"
}

function gotre::log::info {
  gotre::log::_log info "$@"
}

function gotre::log::error {
  gotre::log::_log error "$@"
}
