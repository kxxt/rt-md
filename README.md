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

## License

The code in this repository (except 3rdparty datasets and code inside `3dparty`)
are licensed under GPL-3.0-or-later license.
