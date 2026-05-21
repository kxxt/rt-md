#!/bin/bash

set -e

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

source "$SCRIPT_DIR/scrape.sh"

cd "$SCRIPT_DIR/.."


extract_qps() {
    awk '/QPS: / {
        if (match($0, /QPS: ([0-9]*\.?[0-9]+?)/, x))
        qps = x[1]
    }
    
    END {
        printf "%s", qps;
    }' "$1"
}

extract_utime() {
    awk '/User time \(seconds\): / {
        if (match($0, /User time \(seconds\): ([0-9]*\.?[0-9]+?)/, x))
        qps = x[1]
    }
    
    END {
        printf "%s", qps;
    }' "$1"
}


extract_mrss() {
    awk '/Maximum resident set size \(kbytes\): / {
        if (match($0, /Maximum resident set size \(kbytes\): ([0-9]*\.?[0-9]+?)/, x))
        qps = x[1]
    }
    
    END {
        printf "%s", qps;
    }' "$1"
}

for method in bfcms uniqd ibhh; do
    echo "Method: $method"
    qps=()
    mrss=()
    utime=()
    for i in $(seq 1 5); do
        qps+=("$(extract_qps reports/bench/$method-$i)")
        mrss+=("$(extract_mrss reports/bench/$method-$i)")
        utime+=("$(extract_utime reports/bench/$method-$i)")
    done
    python3 -c 'import sys; values=sys.argv[1:]; print(f"Average of QPS: {sum(float(value) for value in values) / len(values)}")' "${qps[@]}"
    python3 -c 'import sys; values=sys.argv[1:]; print(f"Average of Max. RSS: {sum(float(value) for value in values) / len(values)}")' "${mrss[@]}"
    python3 -c 'import sys; values=sys.argv[1:]; print(f"Average of User time: {sum(float(value) for value in values) / len(values)}")' "${utime[@]}"
done