#!/bin/bash

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

source "$SCRIPT_DIR/vars.sh"

# Build the project
cargo build --release

reports_dir="reports/additive"
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

datasets=()

if [[ -n "$DATASET_ZIZA" ]]; then
    datasets+=(ziza)
fi

if [[ -n "$DATASET_MIXED" ]]; then
    datasets+=(gt)
fi

# Usage: eval_on_dataset dataset method
function additive_on_dataset() {
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
        # none
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet --skip-popularity-allowlist --skip-peacetime-allowlist --skip-internal-allowlist --ablation-rdns-special \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}-none" &
        # popularity allow list
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet --skip-peacetime-allowlist --skip-internal-allowlist --ablation-rdns-special \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}-popularity" &
        # peacetime allow list
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet --skip-popularity-allowlist --skip-internal-allowlist --ablation-rdns-special \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}-peacetime" &
        # internal allow list
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet --skip-popularity-allowlist --skip-peacetime-allowlist --ablation-rdns-special \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}-internal" &
        # rdns special
        target/release/dns-exf-detect -t "${tuned_thresholds[$i]}" -R 120000 --quiet --skip-popularity-allowlist --skip-peacetime-allowlist --skip-internal-allowlist \
            --dataset "$dataset_path" --method "$real_method" eval > "$reports_dir/$method-$dataset-afpr-${ACCEPTABLE_FPRS[$i]}-rdns" &
    done
}

mkdir -p "$reports_dir"

for method in bfcms; do
    for dataset in "${datasets[@]}"; do
        parse_tuned_thresholds "$reports_dir/../comparision/$method-$dataset-tuned"
        additive_on_dataset $dataset $method
    done
done

wait
