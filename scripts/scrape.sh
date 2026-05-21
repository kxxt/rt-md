#!/bin/bash

# scrape dir values
function scrape_client_report() {
    echo "R, TP, FP"
    reports_dir="$1"
    shift
    local r tp fp
    for r in $@;
    do
        lines="$(tail -n 4 "$reports_dir/exp-$r.report")"
        tp=$(awk '$1=="P"{print $2}' <<< "$lines")
        fp=$(awk '$1=="P"{print $3}' <<< "$lines")
        echo "$r, $tp, $fp"
    done
}
