#!/bin/bash

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

source "$SCRIPT_DIR/vars.sh"

afpr_args=()
for v in "${ACCEPTABLE_FPRS[@]}"; do
  afpr_args+=(--acceptable-fpr "$v")
done

# Build the project
cargo build --release

# Tune the thresholds
reports_dir=reports/comparision
mkdir -p "$reports_dir"

if [[ -n "$DATASET_ZIZA" ]]; then
  # Ziza
  target/release/dns-exf-detect -t 0.6 -R 120000 --quiet --dataset datasets/ziza/config.toml --method bfcms tune "${afpr_args[@]}" > "$reports_dir"/bfcms-ziza-tuned &
  target/release/dns-exf-detect -t 0.2 -R 120000 --quiet --dataset datasets/ziza/config.toml --method uniqd tune "${afpr_args[@]}" > "$reports_dir"/uniqd-ziza-tuned &
  target/release/dns-exf-detect -t 0.6 -R 120000 --quiet --dataset datasets/ziza/config.toml --method ibhh  tune "${afpr_args[@]}" > "$reports_dir"/ibhh-ziza-tuned  &
fi

if [[ -n "$DATASET_MIXED" ]]; then
  # Graph Tunnel
  target/release/dns-exf-detect -t 0.6 -R 120000 --quiet --dataset datasets/graph-tunnel/dataset.toml --method bfcms tune "${afpr_args[@]}" > "$reports_dir"/bfcms-gt-tuned &
  target/release/dns-exf-detect -t 0.2 -R 120000 --quiet --dataset datasets/graph-tunnel/dataset.toml --method uniqd tune "${afpr_args[@]}" > "$reports_dir"/uniqd-gt-tuned &
  target/release/dns-exf-detect -t 0.6 -R 120000 --quiet --dataset datasets/graph-tunnel/dataset.toml --method ibhh  tune "${afpr_args[@]}" > "$reports_dir"/ibhh-gt-tuned  &
fi

if [[ -z "$DATASET_MIXED" ]] && [[ -z "$DATASET_ZIZA" ]]; then
  echo "You must set at least one of DATASET_MIXED and DATASET_ZIZA to run this experiment"
fi

wait
