{
  description = "RT-MD artifact evaluation environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      mkArtifacts =
        system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs { inherit system overlays; };
          lib = pkgs.lib;

          rustToolchain = pkgs.rust-bin.stable."1.90.0".default;
          rustPlatform = pkgs.makeRustPlatform {
            cargo = rustToolchain;
            rustc = rustToolchain;
          };

          pythonEnv = pkgs.python313.withPackages (
            ps: [
              ps.dpkt
              ps.ijson
              ps.matplotlib
              ps."more-itertools"
              ps.numpy
              ps.pandas
              ps.pyarrow
              ps.rich
              ps.scapy
              ps.seaborn
              ps.tldextract
            ]
          );

          rubyEnv = pkgs.ruby.withPackages (ps: [ ps.public_suffix ]);

          rEnv = pkgs.rWrapper.override {
            packages = with pkgs.rPackages; [
              dplyr
              ggplot2
              ggpubr
              patchwork
              readr
              stringr
            ];
          };

          rtmdSrc = lib.cleanSourceWith {
            src = ./.;
            filter =
              path: type:
              let
                rel = lib.removePrefix "${toString ./.}/" (toString path);
              in
              !(
                lib.hasPrefix ".git/" rel
                || lib.hasPrefix ".venv/" rel
                || lib.hasPrefix "target/" rel
                || lib.hasPrefix "result" rel
              );
          };

          rtmdPackage = rustPlatform.buildRustPackage {
            pname = "rt-md";
            version = "0.1.0";
            src = rtmdSrc;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkgs.pkg-config ];
            buildInputs = [ pkgs.openssl ];
            doCheck = false;
          };

          commonTools = [
            pkgs.bashInteractive
            pkgs.cacert
            pkgs.coreutils
            pkgs.curl
            pkgs.findutils
            pkgs.gawk
            pkgs.gcc
            pkgs.git
            pkgs.gnugrep
            pkgs.gnumake
            pkgs.gnused
            pkgs.jq
            pkgs.openssl
            pkgs.patch
            pkgs.pkg-config
            pkgs.stdenv.cc.cc.lib
            pkgs.sysstat
            pkgs.time
            pkgs.unzip
            pkgs.uv
            pkgs.which
            pkgs.wireshark-cli
            pkgs.wget
            pkgs.zstd
            rustToolchain
            pythonEnv
            rubyEnv
            rEnv
          ];

          rtmdAe = pkgs.writeShellApplication {
            name = "rtmd-ae";
            runtimeInputs = commonTools;
            text = ''
              set -euo pipefail

              export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              export NIX_SSL_CERT_FILE="$SSL_CERT_FILE"
              export GIT_SSL_CAINFO="$SSL_CERT_FILE"
              export UV_PYTHON="${pythonEnv}/bin/python3"
              export UV_LINK_MODE="''${UV_LINK_MODE:-copy}"
              export OPENSSL_DIR="${pkgs.openssl.dev}"
              export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
              export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:''${PKG_CONFIG_PATH:-}"
              export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib:${pkgs.openssl.out}/lib:''${LD_LIBRARY_PATH:-}"
              export MPLCONFIGDIR="''${MPLCONFIGDIR:-/tmp/matplotlib}"

              usage() {
                cat <<'USAGE'
              rtmd-ae <command>

              Commands:
                help                 Show this help.
                shell                Start a shell with the artifact dependencies.
                env                  Print key tool versions.
                setup                Set up the Python virtualenv and build the Rust release binaries.
                smoke                Run setup, then smoke-test Python, R, Ruby, tshark, mergecap, and zstd.
                prepare-open-data    Download and prepare the public Ziza, GraphTunnel, and multi-domain datasets.
                validate-open-data   Check that prepared open-dataset files are present.
                comparison           Run peacetime generation, threshold tuning, comparison, and table export.
                ablation             Run the false-positive-reduction-by-component ablation and plot export.
                sensitivity          Run threshold and reset-interval sensitivity experiments and plot exports.
                bench                Run throughput/resource benchmarks on Ziza, or mixed with DATASET_MIXED=1.
                open-experiments     Run prepare-open-data, comparison, ablation, sensitivity, and bench.
                real-world-help      Print the live-data commands from the artifact instructions.

              Docker example after loading the image:
                docker run --rm -it -v "$PWD:/workspace/rt-md" rtmd-ae:latest smoke
                docker run --rm -it -v "$PWD:/workspace/rt-md" rtmd-ae:latest open-experiments

              The real-world experiment requires a private syslog DNS stream and cannot be run from
              the public artifacts alone.
              USAGE
              }

              repo_root() {
                local dir
                dir="$(pwd)"
                while [[ "$dir" != "/" ]]; do
                  if [[ -f "$dir/Cargo.toml" && -d "$dir/exp" && -d "$dir/datasets" ]]; then
                    echo "$dir"
                    return
                  fi
                  dir="$(dirname "$dir")"
                done
                if [[ -f /workspace/rt-md/Cargo.toml ]]; then
                  echo /workspace/rt-md
                  return
                fi
                echo "Could not find the rt-md repository. Run this from the repo root or mount it at /workspace/rt-md." >&2
                exit 2
              }

              external_dir() {
                local repo="$1"
                if [[ -n "''${RTMD_EXTERNAL_DIR:-}" ]]; then
                  echo "$RTMD_EXTERNAL_DIR"
                elif [[ -d /workspace && -w /workspace ]]; then
                  echo /workspace/rt-md-external
                else
                  echo "''${XDG_CACHE_HOME:-$HOME/.cache}/rt-md-artifact"
                fi
              }

              enter_repo() {
                local repo
                repo="$(repo_root)"
                cd "$repo"
                echo "$repo"
              }

              ensure_python_env() {
                local repo="$1"
                cd "$repo"
                if [[ "''${RTMD_SKIP_UV_SYNC:-0}" != 1 ]]; then
                  uv sync --locked
                fi
                if [[ -x "$repo/.venv/bin/python" ]]; then
                  export VIRTUAL_ENV="$repo/.venv"
                  export PATH="$repo/.venv/bin:$PATH"
                fi
              }

              build_release() {
                local repo="$1"
                cd "$repo"
                cargo build --release --locked
              }

              setup_workspace() {
                local repo="$1"
                ensure_python_env "$repo"
                build_release "$repo"
              }

              require_file() {
                local path="$1"
                if [[ ! -s "$path" ]]; then
                  echo "Missing or empty required artifact file: $path" >&2
                  return 1
                fi
              }

              require_ziza_data() {
                local repo="$1"
                require_file "$repo/datasets/ziza/wt_dataset.csv"
                require_file "$repo/datasets/ziza/pt_dataset.csv"
                require_file "$repo/datasets/ziza/tuning_dataset.csv"
                require_file "$repo/datasets/ziza/client.oracle"
              }

              require_graph_tunnel_data() {
                local repo="$1"
                require_file "$repo/datasets/graph-tunnel/train.csv.zst"
                require_file "$repo/datasets/graph-tunnel/peacetime.csv.zst"
                require_file "$repo/datasets/graph-tunnel/eval.csv.zst"
                require_file "$repo/datasets/graph-tunnel/all_clients"
                require_file "$repo/datasets/graph-tunnel/val_all_domains.list"
              }

              require_multidomain_data() {
                local repo="$1"
                require_file "$repo/datasets/adversial-md/eval.csv.zst"
                require_file "$repo/datasets/adversial-md/client.oracle"
                require_file "$repo/datasets/adversial-md/all_clients"
                require_file "$repo/datasets/adversial-md/val_all_domains.list"
              }

              cmd_env() {
                echo "Rust: $(rustc --version)"
                echo "Cargo: $(cargo --version)"
                echo "Python: $(python3 --version)"
                echo "uv: $(uv --version)"
                echo "Ruby: $(ruby --version)"
                echo "R: $(Rscript --version 2>&1)"
                echo "tshark: $(tshark --version | head -n 1)"
                echo "mergecap: $(mergecap --version | head -n 1)"
                echo "zstd: $(zstd --version)"
              }

              cmd_setup() {
                local repo
                repo="$(enter_repo)"
                setup_workspace "$repo"
                echo "Setup complete: .venv is ready and release binaries were built under target/release."
              }

              cmd_smoke() {
                local repo
                repo="$(enter_repo)"
                setup_workspace "$repo"
                target/release/dns-exf-detect --help >/dev/null
                target/release/rtmd --help >/dev/null
                python3 - <<'PY'
              import dpkt, ijson, matplotlib, more_itertools, numpy, pandas, pyarrow, rich, scapy, seaborn, tldextract
              print("Python imports OK")
              PY
                Rscript -e 'library(ggplot2); library(stringr); library(dplyr); library(readr); library(patchwork); library(ggpubr); cat("R packages OK\n")'
                ruby -e 'require "public_suffix"; puts "Ruby public_suffix OK"'
                command -v tshark >/dev/null
                command -v mergecap >/dev/null
                command -v zstd >/dev/null
                command -v time >/dev/null
                echo "Smoke validation OK"
              }

              prepare_ziza() {
                local repo="$1"
                local ext="$2"
                local ibhh="$ext/ibHH"
                local ibhh_rev="8d8449f4cb6e43f0b47769a0c9b462dcae57dcf7"
                mkdir -p "$ext"
                if [[ ! -d "$ibhh/.git" ]]; then
                  git clone https://github.com/akamai/Information-based-Heavy-Hitters-for-Real-Time-DNS-Exfiltration-Detection "$ibhh"
                  git -C "$ibhh" checkout "$ibhh_rev"
                fi
                if [[ "$(git -C "$ibhh" rev-parse HEAD)" != "$ibhh_rev" ]]; then
                  echo "$ibhh exists but is not at $ibhh_rev; set RTMD_EXTERNAL_DIR to a clean directory." >&2
                  exit 1
                fi
                if git -C "$ibhh" apply --check "$repo/patches/ibHH.patch" >/dev/null 2>&1; then
                  git -C "$ibhh" apply "$repo/patches/ibHH.patch"
                elif git -C "$ibhh" apply --reverse --check "$repo/patches/ibHH.patch" >/dev/null 2>&1; then
                  echo "ibHH patch is already applied."
                else
                  echo "Could not apply $repo/patches/ibHH.patch to $ibhh." >&2
                  echo "Set RTMD_EXTERNAL_DIR to a clean directory or reset the ibHH checkout." >&2
                  git -C "$ibhh" status --short >&2 || true
                  exit 1
                fi
                if ! grep -Fq 'columns_to_extract = ["timestamp", "request", "user_ip"]' "$ibhh/config.py"; then
                  echo "ibHH patch verification failed: config.py does not keep the user_ip column." >&2
                  exit 1
                fi
                if [[ ! -f "$ibhh/DNS Exfiltration Dataset/dataset.csv" ]]; then
                  curl -L --retry 3 -o "$ext/ziza.zip" https://data.mendeley.com/public-api/zip/c4n7fckkz3/download/3
                  unzip -q -o "$ext/ziza.zip" -d "$ibhh"
                fi
                mkdir -p "$ibhh/data"
                cd "$ibhh"
                python3 preprocess_dataset.py
                python3 split_dataset.py
                cp data/*_dataset.csv "$repo/datasets/ziza/"
                cd "$repo/datasets/ziza"
                python3 process.py
              }

              prepare_graph_tunnel() {
                local repo="$1"
                local ext="$2"
                local raw="$ext/graph-tunnel"
                local graph_rev="e3dccb8a9481cc984592bcc6a018a376dc8e5fac"
                mkdir -p "$ext"
                if [[ ! -d "$raw/.git" ]]; then
                  git clone https://github.com/ggyggy666/DNS-Tunnel-Datasets "$raw"
                  git -C "$raw" checkout "$graph_rev"
                fi
                if [[ "$(git -C "$raw" rev-parse HEAD)" != "$graph_rev" ]]; then
                  echo "$raw exists but is not at $graph_rev; set RTMD_EXTERNAL_DIR to a clean directory." >&2
                  exit 1
                fi

                cd "$repo/datasets/background"
                python3 generate-empty-background.py

                cd "$repo/datasets/graph-tunnel"
                ln -sfn "$raw" raw
                ./merge-traffic.sh
                ./translate.rb
                NO_BACKGROUND_DATASET=1 python3 preprocess.py

                cd "$repo"
                cargo run --release --locked --bin unparquet -- datasets/graph-tunnel/train.parquet
                cargo run --release --locked --bin unparquet -- datasets/graph-tunnel/peacetime.parquet
                cargo run --release --locked --bin unparquet -- datasets/graph-tunnel/eval.parquet
                cd "$repo/datasets/graph-tunnel"
                python3 scrape-unique.py eval.parquet client | tee all_clients
                cd "$repo"
                cargo run --release --locked --bin unique-domains -- datasets/graph-tunnel/eval.parquet > datasets/graph-tunnel/val_all_domains.list
              }

              prepare_multidomain() {
                local repo="$1"
                cd "$repo/datasets/adversial-md"
                make
              }

              cmd_prepare_open_data() {
                local repo ext
                repo="$(enter_repo)"
                ext="$(external_dir "$repo")"
                ensure_python_env "$repo"
                build_release "$repo"
                prepare_ziza "$repo" "$ext"
                prepare_graph_tunnel "$repo" "$ext"
                prepare_multidomain "$repo"
                cmd_validate_open_data
              }

              cmd_validate_open_data() {
                local repo
                repo="$(enter_repo)"
                require_ziza_data "$repo"
                require_graph_tunnel_data "$repo"
                require_multidomain_data "$repo"
                echo "Open dataset validation OK"
              }

              cmd_comparison() {
                local repo
                repo="$(enter_repo)"
                ensure_python_env "$repo"
                require_ziza_data "$repo"
                if [[ -n "''${DATASET_MIXED:-}" ]]; then
                  require_graph_tunnel_data "$repo"
                fi
                require_multidomain_data "$repo"
                DATASET_ZIZA=1 exp/peacetime.sh
                DATASET_ZIZA=1 exp/tune.sh
                DATASET_ZIZA=1 DATASET_MULTID_USE_MIXED_TUNING=1 DATASET_MULTID=1 exp/compare.sh
                mkdir -p exports
                env DATASET_ZIZA=1 DATASET_MULTID=1 scripts/compare-table-export.sh | tee exports/comparison-table.txt
              }

              cmd_ablation() {
                local repo
                repo="$(enter_repo)"
                ensure_python_env "$repo"
                require_ziza_data "$repo"
                if [[ -n "''${DATASET_MIXED:-}" ]]; then
                  require_graph_tunnel_data "$repo"
                fi
                DATASET_ZIZA=1 exp/additive.sh
                mkdir -p exports
                env DATASET_ZIZA=1 scripts/additive-export.sh > exports/additive.csv
                Rscript plot/additive.R
              }

              run_default_dataset() {
                if [[ -n "''${DATASET_MIXED:-}" ]]; then
                  "$@"
                else
                  DATASET_ZIZA=1 "$@"
                fi
              }

              cmd_sensitivity() {
                local repo
                repo="$(enter_repo)"
                ensure_python_env "$repo"
                if [[ -n "''${DATASET_MIXED:-}" ]]; then
                  require_graph_tunnel_data "$repo"
                else
                  require_ziza_data "$repo"
                fi
                export NUM_JOBS="''${NUM_JOBS:-4}"
                run_default_dataset exp/uniqd-threshold.sh
                scripts/uniqd-threshold-exp-export.sh
                run_default_dataset exp/bfcms-threshold.sh
                scripts/bfcms-threshold-exp-export.sh
                Rscript plot/uniqd-threshold-sensitivity.R
                Rscript plot/bfcms-threshold-sensitivity.R
                run_default_dataset exp/uniqd-sensitivity.sh
                scripts/uniqd-reset-interval-exp-export.sh
                run_default_dataset exp/bfcms-sensitivity.sh
                scripts/bfcms-reset-interval-exp-export.sh
                Rscript plot/reset-interval-sensitivity.R
              }

              cmd_bench() {
                local repo
                repo="$(enter_repo)"
                ensure_python_env "$repo"
                if [[ -n "''${DATASET_MIXED:-}" ]]; then
                  require_graph_tunnel_data "$repo"
                  TIME_BIN="${pkgs.time}/bin/time" exp/bench.sh
                else
                  require_ziza_data "$repo"
                  if [[ ! -s reports/comparision/bfcms-ziza-tuned || ! -s reports/comparision/uniqd-ziza-tuned || ! -s reports/comparision/ibhh-ziza-tuned ]]; then
                    DATASET_ZIZA=1 exp/tune.sh
                  fi
                  DATASET_ZIZA=1 TIME_BIN="${pkgs.time}/bin/time" exp/bench.sh
                fi
                scripts/bench-report-export.sh | tee exports/bench-report.txt
              }

              cmd_open_experiments() {
                cmd_prepare_open_data
                cmd_comparison
                cmd_ablation
                cmd_sensitivity
                cmd_bench
              }

              cmd_real_world_help() {
                cat <<'USAGE'
              Real-world evaluation requires a private syslog DNS stream.

              Build the live evaluator:
                cargo build --release --bin rtmd
                mkdir -p reports/real
                cd reports/real

              Tune threshold for one day:
                ../../target/release/rtmd -t 0.5 -R 120000 --syslog 127.0.0.1:5150 --duration 86400 tune --acceptable-fpr 0.001

              Generate live peacetime allowlist for one day:
                ../../target/release/rtmd -t 0.5 -R 120000 --syslog 127.0.0.1:5150 --duration 86400 peacetime

              Run online evaluation for three days:
                ../../target/release/rtmd -t 6.975 -R 120000 --syslog 127.0.0.1:5151 --port 5000 --duration $((60 * 60 * 24 * 3)) eval 2>&1 | tee output

              After completion, run the export and plot commands listed in eval.tex.
              USAGE
              }

              cmd="''${1:-help}"
              shift || true
              case "$cmd" in
                help|-h|--help) usage ;;
                shell) exec bash -i ;;
                env) cmd_env "$@" ;;
                setup|build) cmd_setup "$@" ;;
                smoke) cmd_smoke "$@" ;;
                prepare-open-data|prepare) cmd_prepare_open_data "$@" ;;
                validate-open-data|validate-data) cmd_validate_open_data "$@" ;;
                comparison|compare) cmd_comparison "$@" ;;
                ablation|additive) cmd_ablation "$@" ;;
                sensitivity) cmd_sensitivity "$@" ;;
                bench) cmd_bench "$@" ;;
                open-experiments|all-open) cmd_open_experiments "$@" ;;
                real-world-help) cmd_real_world_help "$@" ;;
                *) echo "Unknown command: $cmd" >&2; usage >&2; exit 2 ;;
              esac
            '';
          };

          usrBinEnv = pkgs.runCommand "usr-bin-env" { } ''
            mkdir -p "$out/usr/bin"
            ln -s "${pkgs.coreutils}/bin/env" "$out/usr/bin/env"
          '';

          imageRoot = pkgs.buildEnv {
            name = "rtmd-ae-root";
            paths = commonTools ++ [
              rtmdAe
              usrBinEnv
            ];
            pathsToLink = [
              "/bin"
              "/etc"
              "/usr/bin"
              "/share"
            ];
          };

          dockerImage = pkgs.dockerTools.buildImage {
            name = "rtmd-ae";
            tag = "latest";
            copyToRoot = imageRoot;
            runAsRoot = ''
              mkdir -p /tmp /workspace
              chmod 1777 /tmp
            '';
            config = {
              Entrypoint = [ "/bin/rtmd-ae" ];
              Cmd = [ "help" ];
              WorkingDir = "/workspace/rt-md";
              Env = [
                "HOME=/tmp"
                "PATH=/bin:/usr/bin"
                "SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
                "NIX_SSL_CERT_FILE=/etc/ssl/certs/ca-bundle.crt"
                "GIT_SSL_CAINFO=/etc/ssl/certs/ca-bundle.crt"
                "UV_LINK_MODE=copy"
                "MPLCONFIGDIR=/tmp/matplotlib"
              ];
            };
          };

          toolSmoke = pkgs.runCommand "rtmd-tool-smoke" { nativeBuildInputs = commonTools; } ''
            set -eu
            export MPLCONFIGDIR="$TMPDIR/matplotlib"
            python3 - <<'PY'
            import dpkt, ijson, matplotlib, more_itertools, numpy, pandas, pyarrow, rich, scapy, seaborn, tldextract
            print("Python imports OK")
            PY
            Rscript -e 'library(ggplot2); library(stringr); library(dplyr); library(readr); library(patchwork); library(ggpubr); cat("R packages OK\n")'
            ruby -e 'require "public_suffix"; puts "Ruby public_suffix OK"'
            command -v tshark >/dev/null
            command -v mergecap >/dev/null
            command -v zstd >/dev/null
            command -v time >/dev/null
            touch "$out"
          '';
        in
        {
          inherit
            pkgs
            commonTools
            dockerImage
            rtmdAe
            rtmdPackage
            rustToolchain
            toolSmoke
            ;
        };
    in
    {
      packages = forAllSystems (
        system:
        let
          a = mkArtifacts system;
        in
        {
          default = a.rtmdPackage;
          rtmd = a.rtmdPackage;
          rtmd-ae = a.rtmdAe;
          dockerImage = a.dockerImage;
        }
      );

      apps = forAllSystems (
        system:
        let
          a = mkArtifacts system;
        in
        {
          default = {
            type = "app";
            program = "${a.rtmdAe}/bin/rtmd-ae";
          };
          rtmd-ae = {
            type = "app";
            program = "${a.rtmdAe}/bin/rtmd-ae";
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          a = mkArtifacts system;
          pkgs = a.pkgs;
        in
        {
          default = pkgs.mkShell {
            packages = a.commonTools ++ [ a.rtmdAe ];
            shellHook = ''
              export SSL_CERT_FILE="${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              export NIX_SSL_CERT_FILE="$SSL_CERT_FILE"
              export GIT_SSL_CAINFO="$SSL_CERT_FILE"
              export UV_PYTHON="${pkgs.python313}/bin/python3"
              export UV_LINK_MODE="copy"
              export OPENSSL_DIR="${pkgs.openssl.dev}"
              export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"
              export OPENSSL_INCLUDE_DIR="${pkgs.openssl.dev}/include"
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:''${PKG_CONFIG_PATH:-}"
              export LD_LIBRARY_PATH="${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.zlib}/lib:${pkgs.openssl.out}/lib:''${LD_LIBRARY_PATH:-}"
              export MPLCONFIGDIR="''${MPLCONFIGDIR:-/tmp/matplotlib}"
              echo "RT-MD artifact shell. Run: rtmd-ae help"
            '';
          };
        }
      );

      checks = forAllSystems (
        system:
        let
          a = mkArtifacts system;
        in
        {
          package = a.rtmdPackage;
          tools = a.toolSmoke;
        }
      );
    };
}
