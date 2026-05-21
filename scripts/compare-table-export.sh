#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "$SCRIPT_DIR/scrape.sh"
source "$SCRIPT_DIR/../exp/vars.sh"

set -eo pipefail

datasets=()

if [[ -n "$DATASET_ZIZA" ]]; then
    datasets+=(ziza)
fi

if [[ -n "$DATASET_MIXED" ]]; then
    datasets+=(gt)
fi

if [[ -n "$DATASET_MULTID" ]]; then
    datasets+=(md)
fi

function extract_metrics() {
    awk '
    BEGIN {
        number = "[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?"
    }

    /^Threshold: / {
        if (match($0, "Threshold:[[:space:]]*(" number ")", x))
        ths = x[1]
    }

    /^P[[:space:]]+/ {
        if (NF >= 3 && $1 == "P") { p_a = $2; p_b = $3 }
    }
    /^N[[:space:]]+/ {
        if (NF >= 3 && $1 == "N") { n_a = $2; n_b = $3 }
    }

    /TPR/ {
        if (match($0, "TPR[^=]*=[[:space:]]*(" number ")", m))
        tpr = m[1]
    }

    /FPR/ {
        if (match($0, "FPR[[:space:]]*=[[:space:]]*(" number ")", n))
        fpr = n[1]
    }

    END {
        printf "%s ", ths
        printf "%d ", p_a
        printf "%d ", p_b
        printf "%d ", n_a
        printf "%d ", n_b
        printf "%s ", tpr
        printf "%s ", fpr
    }
    ' "$1"
}

function extract_domain_metrics() {
    awk '
    BEGIN {
        number = "[-+]?([0-9]+([.][0-9]*)?|[.][0-9]+)([eE][-+]?[0-9]+)?"
    }

    /^Threshold: / {
        if (match($0, "Threshold:[[:space:]]*(" number ")", x))
        ths = x[1]
    }

    /^P[[:space:]]+/ {
        if (NF >= 3 && $1 == "P") { p_a = $2; p_b = $3 }
    }
    /^N[[:space:]]+/ {
        if (NF >= 3 && $1 == "N") { n_a = $2; n_b = $3 }
    }

    /TPR/ {
        if (match($0, "TPR[^=]*=[[:space:]]*(" number ")", m))
        tpr = m[1]
    }

    /FPR/ {
        if (match($0, "FPR[[:space:]]*=[[:space:]]*(" number ")", n))
        fpr = n[1]
    }

    /--- Client evaluation report via client oracle ---/ {
        printf "%s ", ths
        printf "%d ", p_a
        printf "%d ", p_b
        printf "%d ", n_a
        printf "%d ", n_b
        printf "%s ", tpr
        printf "%s ", fpr
        exit
    }
    ' "$1"
}


reports_dir="reports/comparision"

echo "AFPR: TPR FPR TTD Threshold"
for method in bfcms uniqd ibhhc; do
    for dataset in "${datasets[@]}"; do
        echo "Method: $method, Dataset: $dataset"
        for afpr in "${ACCEPTABLE_FPRS[@]}"; do
            metrics=($(extract_metrics "$reports_dir/$method-$dataset-afpr-$afpr"))
            #echo "${metrics[@]}"
            echo "$afpr: ${metrics[5]}(${metrics[1]}) ${metrics[6]:-0}(${metrics[2]}) $(( ${metrics[2]} + ${metrics[1]} )) ${metrics[0]}"
        done
    done
done
