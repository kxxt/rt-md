#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir -p -- "$SCRIPT_DIR/../exports/bfcms"

source "$SCRIPT_DIR/scrape.sh"

reset_intervals="$(seq 20000 20000 1800000)"

scrape_client_report reports/bfcms/reset-interval $reset_intervals > exports/bfcms/reset-interval.csv
# scrape_client_report reports/bfcms/reset-interval-no-allowlist $reset_intervals > exports/bfcms/reset-interval-no-allowlist.csv
# scrape_client_report reports/bfcms/reset-interval-no-peacetime $reset_intervals > exports/bfcms/reset-interval-no-peacetime.csv
scrape_client_report reports/bfcms/reset-interval-no-allowlist $reset_intervals > exports/bfcms/reset-interval-no-allowlist.csv
# scrape_client_report reports/bfcms/reset-interval-only-internal-allowlist $reset_intervals > exports/bfcms/reset-interval-only-internal-allowlist.csv
