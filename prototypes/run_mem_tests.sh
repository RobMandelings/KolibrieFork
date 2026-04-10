#!/bin/zsh

set -e  # optional: exit on first error

for cfg in {0..5}; do
  for strat in clone expire refcount; do
    echo "Running $strat with config$cfg"

    cargo run --bin mem_tests -- $strat $cfg

    mkdir -p "analysis/dhat-heap/config${cfg}"
    mv dhat-heap.json "analysis/dhat-heap/config${cfg}/${strat}.json"
  done
done