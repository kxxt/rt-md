#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir -p -- "$SCRIPT_DIR/../exports/bfcms"

source "$SCRIPT_DIR/scrape.sh"
source "$SCRIPT_DIR/../exp/vars.sh"

scrape_client_report reports/bfcms/threshold ${BFCMS_THRESHOLDS[@]} > exports/bfcms/threshold.csv
scrape_client_report reports/bfcms/threshold-woa ${BFCMS_THRESHOLDS[@]} > exports/bfcms/threshold-woa.csv

