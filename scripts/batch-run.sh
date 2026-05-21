#!/bin/bash

key_flag="$1"
shift

NUM_JOBS="${NUM_JOBS:-4}"
RESULTS_DIR="${RESULTS_DIR:-tmp}"
current_jobs=0

mkdir -p -- "$RESULTS_DIR"

cargo build --release

for item in "$@"; do
    if [[ $current_jobs -ge $NUM_JOBS ]]; then
        wait -n # Wait for any one background job to finish
        current_jobs=$((current_jobs - 1))
    fi

    # Start a new job in the background
    target/release/dns-exf-detect "$key_flag" "$item" $EXTRA_ARGS > "$RESULTS_DIR/exp-$item.report" &
    current_jobs=$((current_jobs + 1))
done

wait
