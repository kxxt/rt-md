#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "$SCRIPT_DIR/scrape.sh"

mkdir -p -- "$SCRIPT_DIR/../exports/uniqd"

reset_intervals="$(seq 20000 20000 1800000)"

scrape_client_report reports/uniqd/reset-interval $reset_intervals > exports/uniqd/reset-interval.csv
# scrape_client_report reports/uniqd/reset-interval-no-allowlist $reset_intervals > exports/uniqd/reset-interval-no-allowlist.csv
# scrape_client_report reports/uniqd/reset-interval-no-peacetime $reset_intervals > exports/uniqd/reset-interval-no-peacetime.csv
scrape_client_report reports/uniqd/reset-interval-no-allowlist $reset_intervals > exports/uniqd/reset-interval-no-allowlist.csv
