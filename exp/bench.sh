#!/bin/bash

set -ex

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd "$SCRIPT_DIR/.."

cargo build --release

reports_dir="reports/bench"
comparison_reports_dir="reports/comparision"
mkdir -p "$reports_dir"

bench_afpr="${BENCH_AFPR:-0.001}"

if [[ -n "${DATASET_MIXED:-}" ]] && [[ -n "${DATASET_ZIZA:-}" ]]; then
    echo "Please only specify one dataset" >&2
    exit 1
fi

if [[ -n "${DATASET_MIXED:-}" ]]; then
    dataset="gt"
    dataset_path="datasets/graph-tunnel/dataset.toml"
    # Threshold values from AFPR 0.001
    bfcms_fallback="2.0083"
    uniqd_fallback="1.2833"
    ibhh_fallback="2.6334"
else
    dataset="ziza"
    dataset_path="datasets/ziza/config.toml"
    # Threshold values from AFPR 0.001
    bfcms_fallback="1.1417"
    uniqd_fallback="0.1083"
    ibhh_fallback="1.0333"
fi

find_gnu_time() {
    local candidate
    local -a candidates

    if [[ -n "${TIME_BIN:-}" ]]; then
        candidates=("$TIME_BIN")
    else
        candidates=()
        if candidate="$(type -P gtime 2>/dev/null)"; then
            candidates+=("$candidate")
        fi
        if candidate="$(type -P time 2>/dev/null)"; then
            candidates+=("$candidate")
        fi
        candidates+=("/usr/bin/time" "/bin/time")
    fi

    for candidate in "${candidates[@]}"; do
        if [[ -x "$candidate" ]] && "$candidate" --version 2>&1 | grep -qi "GNU time"; then
            printf '%s\n' "$candidate"
            return 0
        fi
    done

    echo "GNU time is required. Install GNU time or set TIME_BIN to its executable path." >&2
    return 1
}

TIME_BIN="$(find_gnu_time)"

extract_threshold() {
    local label="$1"
    local method="$2"
    local fallback="$3"
    local file="$comparison_reports_dir/$method-$dataset-tuned"
    local value=""

    if [[ -s "$file" ]]; then
        value="$(awk -v label="$label" -v afpr="$bench_afpr" '$1 == label && $2 == afpr ":" { print $3; exit }' "$file")"
    fi

    if [[ -n "$value" ]]; then
        printf '%s\n' "$value"
        return
    fi

    if [[ -n "$fallback" ]]; then
        printf '%s\n' "$fallback"
        return
    fi

    echo "Missing $label $bench_afpr in $file." >&2
    echo "Run DATASET_ZIZA=1 exp/tune.sh before benchmarking the public Ziza dataset." >&2
    exit 1
}

bfcms_threshold="$(extract_threshold TunedThreshold bfcms "$bfcms_fallback")"
uniqd_threshold="$(extract_threshold TunedThreshold uniqd "$uniqd_fallback")"
ibhh_threshold="$(extract_threshold ClientTunedThreshold ibhh "$ibhh_fallback")"

echo "Benchmark dataset: $dataset_path"
echo "Benchmark AFPR: $bench_afpr"

for i in $(seq 1 5); do
    "$TIME_BIN" -v target/release/dns-exf-detect --dataset "$dataset_path" -t "$bfcms_threshold" -R 120000 --method bfcms bench 2> "$reports_dir"/bfcms-$i
    "$TIME_BIN" -v target/release/dns-exf-detect --dataset "$dataset_path" -t "$uniqd_threshold" -R 120000 --method uniqd bench 2> "$reports_dir"/uniqd-$i
    "$TIME_BIN" -v target/release/dns-exf-detect --dataset "$dataset_path" -t "$ibhh_threshold"  -R 120000 --method ibhh  bench 2> "$reports_dir"/ibhh-$i
done
