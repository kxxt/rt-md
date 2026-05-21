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


function extract_metrics() {
    awk '
    /^Threshold: / {
        if (match($0, /Threshold: ([0-9]*\.?[0-9]+?)/, x))
        ths = x[1]
    }

    /^P[[:space:]]+/ {
        if (NF >= 3 && $1 == "P") { p_a = $2; p_b = $3 }
    }
    /^N[[:space:]]+/ {
        if (NF >= 3 && $1 == "N") { n_a = $2; n_b = $3 }
    }

    /TPR/ {
        if (match($0, /TPR[^(]*\([^)]*\)?[[:space:]]*=[[:space:]]*([0-9]*\.?[0-9]+([eE][+-]?[0-9]+)?)/, m))
        tpr = m[1]
        if (match($0, /FPR[[:space:]]*=[[:space:]]*([0-9]*\.?[0-9]+([eE][+-]?[0-9]+)?)/, n))
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
    /^Threshold: / {
        if (match($0, /Threshold: ([0-9]*\.?[0-9]+?)/, x))
        ths = x[1]
    }

    /^P[[:space:]]+/ {
        if (NF >= 3 && $1 == "P") { p_a = $2; p_b = $3 }
    }
    /^N[[:space:]]+/ {
        if (NF >= 3 && $1 == "N") { n_a = $2; n_b = $3 }
    }

    /TPR/ {
        if (match($0, /TPR[^(]*\([^)]*\)?[[:space:]]*=[[:space:]]*([0-9]*\.?[0-9]+([eE][+-]?[0-9]+)?)/, m))
        tpr = m[1]
        if (match($0, /FPR[[:space:]]*=[[:space:]]*([0-9]*\.?[0-9]+([eE][+-]?[0-9]+)?)/, n))
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


reports_dir="reports/additive"

echo "Method, Dataset, AFPR, Added, TP, FP, FPR"
for method in bfcms; do
    for dataset in "${datasets[@]}"; do
        for afpr in "${ACCEPTABLE_FPRS[@]}"; do
            old_metrics=($(extract_metrics "$reports_dir/../comparision/$method-$dataset-afpr-$afpr"))
            echo "$method, $dataset, $afpr, full, ${old_metrics[1]}, ${old_metrics[2]}, ${old_metrics[6]:-0}"
            for ablation in popularity peacetime internal rdns none; do
                metrics=($(extract_metrics "$reports_dir/$method-$dataset-afpr-$afpr-$ablation"))
                echo "$method, $dataset, $afpr, $ablation, ${metrics[1]}, ${metrics[2]}, ${metrics[6]:-0}"
            done
            #echo "${metrics[@]}"
        done
    done
done
