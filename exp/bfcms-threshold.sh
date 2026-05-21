#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "$SCRIPT_DIR/vars.sh"

if [[ -n "$DATASET_MIXED" ]] && [[ -n "$DATASET_ZIZA" ]]; then
  echo "Please only specify one dataset"
fi

if [[ -n "$DATASET_MIXED" ]]; then
    dataset="datasets/graph-tunnel/dataset.toml"
fi

if [[ -n "$DATASET_ZIZA" ]]; then
    dataset="datasets/ziza/config.toml"
fi

env RESULTS_DIR='reports/bfcms/threshold' \
    EXTRA_ARGS="--quiet --dataset=${dataset} -R 120000 --method bfcms eval" \
    ./scripts/batch-run.sh -t ${BFCMS_THRESHOLDS[@]} &

env RESULTS_DIR='reports/bfcms/threshold-woa' \
    EXTRA_ARGS="--quiet --dataset=${dataset} --skip-peacetime-allowlist --skip-popularity-allowlist --skip-internal-allowlist -R 120000 --method bfcms eval" \
    ./scripts/batch-run.sh -t ${BFCMS_THRESHOLDS[@]} &

# env RESULTS_DIR='reports/bfcms/threshold-trust-rdns' \
#     EXTRA_ARGS='--quiet --dataset datasets/graph-tunnel/dataset.toml --trust-rdns -R 120000 --method bfcms eval' \
#     ./scripts/batch-run.sh -t ${BFCMS_THRESHOLDS[@]}

wait