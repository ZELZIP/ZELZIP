#!/usr/bin/env bash

gotre::log::info "Checking todo list files"
glow ./**/TODO.md

gotre::log::info "Cheking inline todo comments"
rg TODO --iglob !TODO.md --iglob !/projects/toolkit/todo.bash
