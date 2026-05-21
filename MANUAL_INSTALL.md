# RT-MD Manual Installation Guide

This document describes how to set up the RT-MD artifact, prepare the datasets,
and run the experiments manually used in the paper.

We highly recommend using the Nix flake or Docker image to save the time for setting up the environment.

The commands below assume that the repository has been cloned and that its
absolute path is stored in `srcdir`:

```bash
export srcdir=/path/to/rt-md
cd "$srcdir"
```

All experiments are expected to run on GNU/Linux. The original experiments were
run primarily on Debian Trixie.

## Artifacts

The following artifacts are needed to reproduce the public parts of the
evaluation:

- RT-MD source code: this repository.
- ibHH source code: used for the Ziza dataset split scripts.
  Repository: <https://github.com/akamai/Information-based-Heavy-Hitters-for-Real-Time-DNS-Exfiltration-Detection>
- Ziza dataset: public DNS exfiltration dataset used for public evaluation.
- GraphTunnel dataset: public DNS tunnel dataset.
  Repository: <https://github.com/ggyggy666/DNS-Tunnel-Datasets>

The following artifact was used in the paper but is not included:

- Campus DNS logs: used as benign background traffic for the mixed dataset and
  as the live stream for the real-world evaluation.

The campus DNS logs cannot be shared because they contain sensitive DNS queries
and host IP addresses. Even anonymized query names or timing features can expose
users or hosts.

## Recommended Setup With Nix/Docker

Please see [README](./README.md) or the *Artifact Appendix* in our paper.

## Manual Environment Setup

If you do not use Nix or Docker, install these dependencies manually.

Required tools:

- Rust toolchain 1.90, installed through <https://rustup.rs/>
- `uv`, from <https://docs.astral.sh/uv/>
- Python 3
- Ruby
- `mergecap`
- `tshark`
- `zstd`
- GNU `time`
- R 4.5.0 or compatible

Required R packages:

- `ggplot2`
- `stringr`
- `dplyr`
- `readr`
- `patchwork`
- `ggpubr`

Set up the Python environment and build the Rust binaries:

```bash
cd "$srcdir"
uv sync
source .venv/bin/activate
cargo build --release
```

## Dataset Preparation

### Ziza Dataset

Clone ibHH:

```bash
cd /path/for/external/artifacts
git clone https://github.com/akamai/Information-based-Heavy-Hitters-for-Real-Time-DNS-Exfiltration-Detection ibHH
cd ibHH
```

Download the Ziza dataset. The Nix helper uses this public archive URL:

```text
https://data.mendeley.com/public-api/zip/c4n7fckkz3/download/3
```

Place `dataset.csv` from the archive at:

```text
DNS Exfiltration Dataset/dataset.csv
```

Patch `config.py` so the source IP column is preserved:

```diff
-columns_to_extract = ["timestamp", "request"]
+columns_to_extract = ["timestamp", "request", "user_ip"]
```

You can also apply the patch from this repository:

```bash
git apply "$srcdir"/patches/ibHH.patch
```

Then preprocess and split the dataset:

```bash
mkdir -p data
python3 -m venv .venv
source .venv/bin/activate
pip install pandas
python3 preprocess_dataset.py
python3 split_dataset.py
```

Copy the generated files into this repository and run the RT-MD Ziza
post-processing step:

```bash
cp data/*_dataset.csv "$srcdir"/datasets/ziza/
cd "$srcdir"
source .venv/bin/activate
cd datasets/ziza
./process.py
```

Expected generated files include:

- `datasets/ziza/wt_dataset.csv`
- `datasets/ziza/pt_dataset.csv`
- `datasets/ziza/tuning_dataset.csv`
- `datasets/ziza/client.oracle`
- `datasets/ziza/tune.client.oracle`

### Mixed Dataset

The mixed dataset combines GraphTunnel malicious traffic with benign background
DNS traffic. The paper used campus DNS logs as the background, but those logs are
not public. You may bring your own background DNS logs.

Clone the GraphTunnel dataset and merge split PCAPs:

```bash
cd "$srcdir"/datasets/graph-tunnel
git clone https://github.com/ggyggy666/DNS-Tunnel-Datasets raw
./merge-traffic.sh
```

Translate PCAPs into DNS log files:

```bash
./translate.rb
```

Place the background dataset at:

```text
datasets/background/background.parquet
```

The expected parquet schema is defined in
`datasets/graph-tunnel/preprocess.py`.

Preprocess GraphTunnel dataset and convert parquet files into RT-MD's compressed CSV
dataset format:

```bash
cd "$srcdir"
source .venv/bin/activate
cd datasets/graph-tunnel
./preprocess.py

cd "$srcdir"
cargo run --release --bin unparquet -- datasets/graph-tunnel/train.parquet
cargo run --release --bin unparquet -- datasets/graph-tunnel/peacetime.parquet
cargo run --release --bin unparquet -- datasets/graph-tunnel/eval.parquet

cd datasets/graph-tunnel
./scrape-unique.py eval.parquet client | tee all_clients
env -C ../.. cargo run --release --bin unique-domains -- datasets/graph-tunnel/eval.parquet > val_all_domains.list
```

Expected generated files include:

- `datasets/graph-tunnel/train.csv.zst`
- `datasets/graph-tunnel/peacetime.csv.zst`
- `datasets/graph-tunnel/eval.csv.zst`
- `datasets/graph-tunnel/all_clients`
- `datasets/graph-tunnel/val_all_domains.list`

### Multi-Domain Exfiltration Dataset

Generate the adversarial multi-domain dataset:

```bash
cd "$srcdir"
source .venv/bin/activate
cd datasets/adversial-md
make
```

Expected generated files include:

- `datasets/adversial-md/eval.csv.zst`
- `datasets/adversial-md/client.oracle`
- `datasets/adversial-md/all_clients`
- `datasets/adversial-md/val_all_domains.list`

### Peacetime Allowlists

Generate peacetime allowlists:

```bash
cd "$srcdir"
DATASET_ZIZA=1 DATASET_MIXED=1 exp/peacetime.sh
```

To run only one dataset, remove the other environment variable. For example:

```bash
DATASET_ZIZA=1 exp/peacetime.sh
DATASET_MIXED=1 exp/peacetime.sh
```

## Sensitivity Analysis

In our paper, we ran the sensitivity analysis experiments on the mixed dataset.
However, since that dataset is not fully public, the commands here run the experiments
on Ziza dataset instead. 
If you put together a mixed dataset, replace `DATASET_ZIZA=1` with `DATASET_MIXED=1`
in the following commands to use it.

### Threshold Sensitivity

Run UniqD threshold sensitivity:

```bash
cd "$srcdir"
DATASET_ZIZA=1 NUM_JOBS=100 exp/uniqd-threshold.sh
DATASET_ZIZA=1 scripts/uniqd-threshold-exp-export.sh
```

Outputs:

- raw reports: `reports/uniqd/threshold`
- raw reports without allowlists: `reports/uniqd/threshold-woa`
- summary report: `exports/uniqd/threshold.csv`
- summary report without allowlists: `exports/uniqd/threshold-woa.csv`

Run RT-MD threshold sensitivity:

```bash
cd "$srcdir"
DATASET_ZIZA=1 NUM_JOBS=64 exp/bfcms-threshold.sh
DATASET_ZIZA=1 scripts/bfcms-threshold-exp-export.sh
```

Outputs:

- raw reports: `reports/bfcms/threshold`
- raw reports without allowlists: `reports/bfcms/threshold-woa`
- summary report: `exports/bfcms/threshold.csv`
- summary report without allowlists: `exports/bfcms/threshold-woa.csv`

Create threshold sensitivity plots:

```bash
DATASET_ZIZA=1 Rscript plot/uniqd-threshold-sensitivity.R
DATASET_ZIZA=1 Rscript plot/bfcms-threshold-sensitivity.R
```

Outputs:

- `plot/uniqd-threshold.pdf`
- `plot/bfcms-threshold.pdf`

### Reset-Interval Sensitivity

Run UniqD reset-interval sensitivity:

```bash
cd "$srcdir"
DATASET_ZIZA=1 NUM_JOBS=100 exp/uniqd-sensitivity.sh
DATASET_ZIZA=1 scripts/uniqd-reset-interval-exp-export.sh
```

Outputs:

- raw reports: `reports/uniqd/reset-interval`
- raw reports without allowlists: `reports/uniqd/reset-interval-no-allowlist`
- exports: `exports/uniqd/reset-interval.csv`
- exports without allowlists: `exports/uniqd/reset-interval-no-allowlist.csv`

Run RT-MD reset-interval sensitivity:

```bash
cd "$srcdir"
DATASET_ZIZA=1 NUM_JOBS=64 exp/bfcms-sensitivity.sh
DATASET_ZIZA=1 scripts/bfcms-reset-interval-exp-export.sh
```

Outputs:

- raw reports: `reports/bfcms/reset-interval`
- raw reports without allowlists: `reports/bfcms/reset-interval-no-allowlist`
- exports: `exports/bfcms/reset-interval.csv`
- exports without allowlists: `exports/bfcms/reset-interval-no-allowlist.csv`

Create the combined reset-interval plot:

```bash
Rscript plot/reset-interval-sensitivity.R
```

The current script writes:

- `plot/reset-interval.pdf`
- `plot/reset-interval-fp.pdf`
- `plot/reset-interval-tp.pdf`

## Comparison Experiment

### Threshold Tuning

Tune thresholds for the acceptable false-positive rates:

```bash
cd "$srcdir"
DATASET_ZIZA=1 exp/tune.sh
```

To tune only one dataset, remove the other dataset variable.

### Evaluation

Run the comparison experiment:

```bash
cd "$srcdir"
DATASET_ZIZA=1 DATASET_MULTID_USE_MIXED_TUNING=1 DATASET_MULTID=1 exp/compare.sh
```

To exclude a dataset, remove the corresponding environment variable.

By default, the multi-domain exfiltration dataset uses thresholds tuned on the
mixed dataset even when it is not available to match the exact setting in our paper.
This is possible because we provide the tuning results on the mixed dataset in
`reports/comparision/{bfcms,ibhh,uniqd}-gt-tuned`.
To use the thresholds tuned on Ziza datasets, remove `DATASET_MULTID_USE_MIXED_TUNING=1`.
To include the mixed dataset in this experiment, add `DATASET_MIXED=1`.

Generate the comparison table:

```bash
env DATASET_ZIZA=1 DATASET_MULTID=1 scripts/compare-table-export.sh
```

## False-Positive Reduction by Component

Run the RT-MD ablation study:

```bash
cd "$srcdir"
DATASET_ZIZA=1 exp/additive.sh
```

Aggregate results:

```bash
env DATASET_ZIZA=1 scripts/additive-export.sh > exports/additive.csv
```

Create the plot:

```bash
DATASET_ZIZA=1 Rscript plot/additive.R
```

Output:

- `plot/additive.pdf`

## Throughput and Resource Usage

This experiment depends on the tuned thresholds from the comparison experiment.
We ran this experiment on the mixed dataset in our paper.
The commands below run it on the public Ziza dataset.

Run the benchmark:

```bash
cd "$srcdir"
DATASET_ZIZA=1 exp/bench.sh
```

Export benchmark results:

```bash
DATASET_ZIZA=1 scripts/bench-report-export.sh
```

## Real-World Evaluation

The real-world evaluation requires a private live DNS syslog stream and cannot
be reproduced from the public artifacts alone.

Build the live evaluator and enter the report directory:

```bash
cd "$srcdir"
cargo build --release --bin rtmd
mkdir -p reports/real
cd reports/real
```

The resource-usage scripts use `pidstat` from `sysstat` with JSON output. If
your distribution does not package a version with JSON support, build it from
source:

```bash
git clone https://github.com/sysstat/sysstat
cd sysstat
./configure
make -j"$(nproc)"
export PATH="$PWD:$PATH"
```

### Live DNS Syslog Format

RT-MD accepts syslog input from a UDP socket. The syslog payload should be a
space-separated key-value record, for example:

```text
q_time=2025-11-12T10:59:36.205934 a_time=2025-11-12T10:59:36.227074 src=<IPADDR> sport=12345 dst=8.8.8.8 tid=23456 q_name=example.com q_type=AAAA a_ip=127.0.0.1 a_cname= error=
```

### Threshold Tuning

Tune a threshold on one day of live data:

```bash
../../target/release/rtmd -t 0.5 -R 120000 \
  --syslog 127.0.0.1:5150 --duration 86400 tune \
  --acceptable-fpr 0.001
```

The tuned threshold is printed in the program output.

### Peacetime Allowlist Generation

Generate a peacetime allowlist from one day of live data:

```bash
../../target/release/rtmd -t 0.5 -R 120000 \
  --syslog 127.0.0.1:5150 --duration 86400 peacetime
```

### Online Evaluation

Run online evaluation for three days:

```bash
../../target/release/rtmd -t 6.975 -R 120000 \
  --syslog 127.0.0.1:5150 --port 5000 \
  --duration $((60 * 60 * 24 * 3)) \
  eval 2>&1 | tee output
```

The dashboard is served at:

```text
http://127.0.0.1:5000
```

### Real-World Plot Exports

After the live run finishes, aggregate resource usage and throughput:

```bash
cd "$srcdir"
source .venv/bin/activate
scripts/resource-usage-export.py reports/real/pidstat.json
scripts/throughput-export.py reports/real/output
```

Create the human-operations plot:

```bash
scripts/human-ops-export.py reports/real/output
```

Output:

- `plot/real-world-human-ops.pdf`

Create the alert-series plot:

```bash
scripts/alert-series-export.py reports/real/alerts.jsonl
```

Output:

- `plot/real-world-alerts.pdf`

Create the resource and throughput plot:

```bash
plot/real-world-throughput-and-resource.py
```

Output:

- `plot/real-world-throughput-and-resource.pdf`
