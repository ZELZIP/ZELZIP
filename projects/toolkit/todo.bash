#!/usr/bin/env bash

gotre::log::info "Checking todo list files"
for todo_file in ./**/TODO.md; do
  glow "$todo_file"
done

gotre::log::info "Cheking inline todo comments"
rg TODO --iglob !TODO.md --iglob !/projects/toolkit/todo.bash
