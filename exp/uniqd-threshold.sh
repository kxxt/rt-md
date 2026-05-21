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

env RESULTS_DIR='reports/uniqd/threshold' \
    EXTRA_ARGS="--dataset=${dataset} -R 120000 --method uniqd eval" \
    ./scripts/batch-run.sh -t ${UNIQD_THRESHOLDS[@]}

env RESULTS_DIR='reports/uniqd/threshold-woa' \
    EXTRA_ARGS="--dataset=${dataset} -R 120000 --skip-peacetime-allowlist --skip-popularity-allowlist --skip-internal-allowlist --method uniqd eval" \
    ./scripts/batch-run.sh -t ${UNIQD_THRESHOLDS[@]}

# env RESULTS_DIR='reports/uniqd/threshold-r60' \
#     EXTRA_ARGS='--dataset datasets/graph-tunnel/dataset.toml -R 60000 --trust-rdns --method uniqd eval' \
#     ./scripts/batch-run.sh -t ${UNIQD_THRESHOLDS[@]}
