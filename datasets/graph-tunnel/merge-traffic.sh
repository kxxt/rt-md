#!/bin/bash

set -euo pipefail

mergecap -w raw/normal/normal.pcap raw/normal/normal/*.pcap
mergecap -w raw/tunnel/dns2tcp-key.pcap raw/tunnel/dns2tcp-key/*.pcap
mergecap -w raw/wildcard.pcap raw/wildcard/*.pcap