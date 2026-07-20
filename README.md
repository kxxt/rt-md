# RT-MD

This repo contains the code for our CCS'26 paper *RT-MD: Host-Centric Real-Time Detection of Multi-Domain DNS Data Exfiltration*.

The Open Science appendix and Artifact appendix of our paper contain detailed instructions for setup and evaluation.

In the code, RT-MD is internally codenamed `bfcms`.

## Artifact Evaluation With Nix

This repository provides a Nix flake that pins the Rust 1.90.0 toolchain and
the Python, Ruby, R, packet-processing, compression, and benchmarking tools
used by the artifact scripts.

The real-world experiment depends on a private live DNS syslog stream and is
not reproducible from the public artifacts alone. Run `rtmd-ae real-world-help`
for the exact commands from the artifact instructions.

To enter the artifact evaluation environment, run the following command.

```bash
nix develop
```

If you don't have Nix installed, you can also use the prebuilt docker image in the artifacts
with the following commands.

```bash
docker load -i PATH/TO/docker-image-rtmd-ae.tar.gz
docker run -it -v "$PWD:/workspace/rt-md" \
  rtmd-ae:latest shell
```

The `rtmd-ae` experiment commands default to public datasets. To opt into
mixed-dataset experiments when the private data is available, set
`DATASET_MIXED=1` before the command.

Common commands:

```bash
rtmd-ae help
rtmd-ae setup
rtmd-ae smoke
rtmd-ae prepare-open-data
rtmd-ae ablation
rtmd-ae compare
rtmd-ae sensitivity
```

To build the Docker image, run:

```bash
nix build .#dockerImage
docker load < result
docker run --rm -it -v "$PWD:/workspace/rt-md" rtmd-ae:latest shell
```

In case you don't use nix or docker, we provide instructions for manually setting up the environment in [MANUAL_INSTALL.md](./MANUAL_INSTALL.md).

## Repository Structure

- `src`: Rust implementation of RT-MD, UniqD and ibHH.
- `allowlist`: Global popularity-based allowlist and internal allowlist.
- `psl`: Configurable suffix list, which is a superset of Public Suffix List.
- `datasets`: The datasets and associated scripts.
- `exp`: Scripts for executing the experiments.
- `scripts`: Auxiliary scripts.
- `reports`: Raw experiment outputs.
- `exports`: High-level experiment results.
- `plot`: Scripts for generating plots (and the plots).
- `frontend`: The frontend of the alert management dashboard used in real-world evaluation.
- `3dparty`: third party source code.

## Real-World Evaluation

To perform the real-world evaluation, a real-time DNS query log source is required.

### Additional Environment Setup

Assuming the source directory of this repo is `$srcdir`,
first, build the project and enter the directory for conducting real-world evaluation.

```bash
cd "$srcdir"
cargo build --release --bin rtmd
mkdir -p reports/real && cd $_
```

We use the `pidstat` command from `sysstat` to export the CPU and memory usage of our detection system in JSON format.
However, most Linux distributions do not package `sysstat` with JSON output-formatting capability.
Thus, this software needs to be built from source.

```bash
git clone https://github.com/sysstat/sysstat
./configure
make -j$(nproc)
export PATH="$PWD:$PATH"
```

### Real-Time DNS Log Stream Format

We expect the real-time DNS log stream to be made available in syslog format.
Our program accepts syslog input from a UDP socket.
The commands listed in this section use `127.0.0.1:5150` as an example.
The syslog content should conform to a simple space-separated key-value format, as demonstrated by the following example:

```text
q_time=2025-11-12T10:59:36.205934 a_time=2025-11-12T10:59:36.227074 src=<IPADDR> sport=12345 dst=8.8.8.8 tid=23456 q_name=example.com q_type=AAAA a_ip=127.0.0.1 a_cname= error=
```

### Threshold Tuning

To tune the threshold on live data for one day, run the following command:

```bash
../../target/release/rtmd -t 0.5 -R 120000 \
   --syslog 127.0.0.1:5150 --duration 86400 tune \
   --acceptable-fpr 0.001
```

The threshold and acceptable FPR used for tuning can be adjusted.
The tuned threshold will be available in the program output.

### Peacetime Allowlist Generation

To generate the peacetime allowlist from live data for one day, run the following command:

```bash
../../target/release/rtmd -t 0.5 -R 120000 \
   --syslog 127.0.0.1:5150 --duration 86400 peacetime
```

### Online Evaluation

To start the online evaluation for three days, run the following command:

```bash
../../target/release/rtmd -t 6.975 -R 120000 \
   --syslog 127.0.0.1:5151 --port 5000 \
   --duration $((60 * 60 * 24 * 3)) \
   eval 2>&1 | tee output
```

As shown in the image, the management dashboard will be available at http://127.0.0.1:5000 once the evaluation starts.
A human operator is expected to take action to eliminate false alerts and address true-positive alerts.

![RT-MD Detection Dashboard](plot/dashboard.png)

### Plots

After the experiment finishes, aggregate the results by running the following commands:

```bash
cd "$srcdir"
source .venv/bin/activate

# Collect resource usage
scripts/resource-usage-export.py \
  reports/real/pidstat.json

# Collect throughput
scripts/throughput-export.py reports/real/output
```

Then create the plots.

The figure showing the amount of human labor, Figure 6(a) in the paper, can be created by running:

```bash
scripts/human-ops-export.py reports/real/output

# Output:
# plot/real-world-human-ops.pdf
```

The figure showing the number of remaining/cumulative alerts over time, Figure 6(b), can be created by running:

```bash
scripts/alert-series-export.py \
   reports/real/alerts.jsonl

# Output:
# plot/real-world-alerts.pdf
```

The figure showing resource usage and throughput, Figure 6(c), can be created by running:

```bash
plot/real-world-throughput-and-resource.py

# Output:
# plot/real-world-throughput-and-resource.pdf
```


## License

The code in this repository (except 3rdparty datasets and code inside `3dparty`)
are licensed under GPL-3.0-or-later license.
