#!/bin/bash

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

source "$SCRIPT_DIR/vars.sh"

# Build the project
cargo build --release

datasets=()

if [[ -n "$DATASET_ZIZA" ]]; then
    datasets+=(ziza)
fi

if [[ -n "$DATASET_MIXED" ]]; then
    datasets+=(gt)
fi

reports_dir="reports/comparision"
tuned_thresholds=()

function parse_tuned_thresholds() {
    tuned_thresholds=()
    while read -r fpr value; do
        tuned_thresholds+=("${value}")
    done < <(awk '/^TunedThreshold / {sub(":", "", $2); print $2, $3}' "$1")
}

function parse_client_tuned_thresholds() {
    tuned_thresholds=()
    while read -r fpr value; do
        tuned_thresholds+=("${value}")
    done < <(awk '/^ClientTunedThreshold / {sub(":", "", $2); print $2, $3}' "$1")
}

# Usage: eval_on_dataset dataset method
function eval_on_dataset() {
    local dataset_path
    local dataset="$1"
    local method="$2"
    local real_method="$2"
    case "$dataset" in
        ziza)
            dataset_path=datasets/ziza/config.toml
            ;;
        gt)
            dataset_path=datasets/graph-tunnel/dataset.toml
            ;;
        md)
            dataset_path=datasets/adversial-md/dataset.toml
            ;;
        *)
            echo "Unrecognized dataset!"
            exit 1
            ;;
    esac
    case "$method" in
        ibhhc)
            real_method="ibhh"
            ;;
    esac
    local i n=${#ACCEPTABLE_FPRS[@]}
    for (( i=0; i<n; i++ )); do
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}" &
    done
}

for method in bfcms uniqd; do
    for dataset in "${datasets[@]}"; do
        parse_tuned_thresholds "$reports_dir/$method-$dataset-tuned"
        eval_on_dataset $dataset $method
    done
    if [[ -n "$DATASET_MULTID" ]]; then
        # We use the tuned threshold from gt for adversial-md dataset
        if [[ -n "$DATASET_MULTID_USE_MIXED_TUNING" ]]; then 
            parse_tuned_thresholds "$reports_dir/$method-gt-tuned"
        elif [[ -n "$DATASET_MIXED" ]]; then 
            parse_tuned_thresholds "$reports_dir/$method-gt-tuned"
        elif [[ -n "$DATASET_ZIZA" ]]; then
            parse_tuned_thresholds "$reports_dir/$method-ziza-tuned"
        else
            echo "No other dataset specified. Aborting"
            exit 1
        fi
        eval_on_dataset md $method
    fi
done

# ibHH (client tuned)
for dataset in "${datasets[@]}"; do
    parse_client_tuned_thresholds "$reports_dir/ibhh-$dataset-tuned"
    eval_on_dataset $dataset ibhhc
done
if [[ -n "$DATASET_MULTID" ]]; then
    # We use the tuned threshold from gt for adversial-md dataset by default
    if [[ -n "$DATASET_MULTID_USE_MIXED_TUNING" ]]; then 
        parse_client_tuned_thresholds "$reports_dir/ibhh-gt-tuned"
    elif [[ -n "$DATASET_MIXED" ]]; then 
        parse_client_tuned_thresholds "$reports_dir/ibhh-gt-tuned"
    elif [[ -n "$DATASET_ZIZA" ]]; then
        parse_client_tuned_thresholds "$reports_dir/ibhh-ziza-tuned"
    else
        echo "No other dataset specified. Aborting"
        exit 1
    fi
    eval_on_dataset md ibhhc
fi

wait
