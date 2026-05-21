#!/bin/bash

threshold=0.5

if [[ -n "$DATASET_MIXED" ]] && [[ -n "$DATASET_ZIZA" ]]; then
  echo "Please only specify one dataset"
fi

if [[ -n "$DATASET_MIXED" ]]; then
    dataset="datasets/graph-tunnel/dataset.toml"
fi

if [[ -n "$DATASET_ZIZA" ]]; then
    dataset="datasets/ziza/config.toml"
fi

env RESULTS_DIR='reports/uniqd/reset-interval' \
    EXTRA_ARGS="--quiet --dataset=${dataset} -t $threshold --method uniqd eval" \
    ./scripts/batch-run.sh -R $(seq 20000 20000 1800000)

# env RESULTS_DIR='reports/uniqd/reset-interval-no-popularity' \
#     EXTRA_ARGS="--quiet --dataset datasets/graph-tunnel/dataset.toml --skip-popularity-allowlist -t $threshold --method uniqd eval" \
#     ./scripts/batch-run.sh -R $(seq 20000 20000 1800000)

# env RESULTS_DIR='reports/uniqd/reset-interval-no-peacetime' \
#     EXTRA_ARGS="--quiet --dataset datasets/graph-tunnel/dataset.toml --skip-peacetime-allowlist -t $threshold --method uniqd eval" \
#     ./scripts/batch-run.sh -R $(seq 20000 20000 1800000)

env RESULTS_DIR='reports/uniqd/reset-interval-no-allowlist' \
    EXTRA_ARGS="--quiet --dataset=${dataset} --skip-popularity-allowlist --skip-peacetime-allowlist -t $threshold --method uniqd eval" \
    ./scripts/batch-run.sh -R $(seq 20000 20000 1800000)
