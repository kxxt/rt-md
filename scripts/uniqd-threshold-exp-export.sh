#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

mkdir -p -- "$SCRIPT_DIR/../exports/uniqd"

source "$SCRIPT_DIR/scrape.sh"
source "$SCRIPT_DIR/../exp/vars.sh"

scrape_client_report reports/uniqd/threshold ${UNIQD_THRESHOLDS[@]} > exports/uniqd/threshold.csv
scrape_client_report reports/uniqd/threshold-woa ${UNIQD_THRESHOLDS[@]} > exports/uniqd/threshold-woa.csv

