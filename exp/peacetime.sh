#!/bin/bash

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

source "$SCRIPT_DIR/vars.sh"

# Build the project
cargo build --release

# Tune the thresholds
reports_dir=reports/comparision
mkdir -p "$reports_dir"

if [[ -n "$DATASET_ZIZA" ]]; then
  # Ziza
  target/release/dns-exf-detect -t 0.5 -R 120000 --quiet --dataset datasets/ziza/config.toml --method bfcms peacetime > "$reports_dir"/bfcms-ziza-pt &
  target/release/dns-exf-detect -t 0.5 -R 120000 --quiet --dataset datasets/ziza/config.toml --method ibhh  peacetime > "$reports_dir"/ibhh-ziza-pt  &
fi

if [[ -n "$DATASET_MIXED" ]]; then
  # Graph Tunnel
  target/release/dns-exf-detect -t 0.5 -R 120000 --quiet --dataset datasets/graph-tunnel/dataset.toml --method bfcms peacetime > "$reports_dir"/bfcms-gt-pt &
  target/release/dns-exf-detect -t 0.5 -R 120000 --quiet --dataset datasets/graph-tunnel/dataset.toml --method ibhh  peacetime > "$reports_dir"/ibhh-gt-pt  &
fi

if [[ -z "$DATASET_MIXED" ]] && [[ -z "$DATASET_ZIZA" ]]; then
  echo "You must set at least one of DATASET_MIXED and DATASET_ZIZA to run this experiment"
fi

wait
