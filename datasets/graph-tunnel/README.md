# GraphTunnel Dataset

The normal traffic's duration is roughly 1 day 2hours.
While all the malicious traffic lasts 1 day, 15:19:32.484681 together.

Because the normal traffic from this dataset contains only one client,
we use another source of DNS traffic collected from our campus as benign
normal traffic.

We partition the time range into 9:2:13.5

There are 25 kinds of malicious tunnels,
We map the queries in them to a unique source IP address and the domain
to a unique domain to test RT-MD and ibHH's performance against individual
exfiltration tools.

We mix the malicious traffic back into the normal traffic by

At hour 11:00, we start the crossEndPoint exfiltration,
which contains 5 pcaps AndIodine lasting for 2:21.

At 13:30, roughly 10 minutes after crossEndPoint exfiltration,
we start another 5 exfiltration campaigns, DNS-shell, dnscat2-\* and dnspot,
which ends at 15:30.

At 15:45, we start iodine exfiltration, which ends at 16:47.

At 17:00, we start the rest of the exfiltration except cobalstrike and tuns,
which ends at 21:28.

At 21:40, we start cobalstrike and tuns, which ends at 23:53.

Duration statistics:

    Duration of crossEndPoint/AndIodine-CNAME.pcap: 1:22:26.497724
    Duration of crossEndPoint/AndIodine-MX.pcap: 1:01:27.416011
    Duration of crossEndPoint/AndIodine-NULL.pcap: 0:35:38.419973
    Duration of crossEndPoint/AndIodine-SRV.pcap: 1:03:09.925185
    Duration of crossEndPoint/AndIodine-TXT.pcap: 2:20:18.515813
    Duration of normal/normal.pcap: 1 day, 0:25:35.767402
    Duration of tunnel/DNS-shell.pcap: 1:49:58.176468
    Duration of tunnel/dnscat2-cname.pcap: 1:32:35.023253
    Duration of tunnel/dnscat2-mx.pcap: 2:24:21.932968
    Duration of tunnel/dnscat2-txt.pcap: 1:19:49.856549
    Duration of tunnel/dnspot.pcap: 2:00:54.259778
    Duration of tunnel/iodine-NULL.pcap: 0:39:48.677316
    Duration of tunnel/iodine-a.pcap: 0:45:29.571266
    Duration of tunnel/iodine-cname.pcap: 1:00:07.178861
    Duration of tunnel/iodine-mx.pcap: 1:02:49.018176
    Duration of tunnel/iodine-private.pcap: 0:26:17.177134
    Duration of tunnel/iodine-srv.pcap: 0:44:55.562405
    Duration of tunnel/iodine-txt.pcap: 0:33:55.058985
    Duration of tunnel/tuns.pcap: 1:42:21.997508
    Duration of tunnel/dns2tcp-key.pcap: 3:11:28.523398
    Duration of unkownTunnel/cobalstrike.pcap: 2:12:47.393179
    Duration of unkownTunnel/dns2tcp-key.pcap: 3:11:28.523398
    Duration of unkownTunnel/dns2tcp-txt.pcap: 0:53:36.891867
    Duration of unkownTunnel/ozymandns.pcap: 0:44:00.044785
    Duration of unkownTunnel/tcp-over-dns-CNAME.pcap: 4:28:26.605419
    Duration of unkownTunnel/tcp-over-dns-TXT.pcap: 2:11:20.237262
    Duration of wildcard.pcap: 1:00:21.784627
    Malicious duration: 1 day, 15:19:32.484681
    Normal duration: 1 day, 0:25:35.767402
    Wildcard duration: 1:00:21.784627


## Prerequisitos

Commandline tools like `tshark` and `mergecap` are required for preprecessing
this dataset.

## Initialization

Clone the GraphTunnel dataset by:

```bash
git clone https://github.com/ggyggy666/DNS-Tunnel-Datasets raw
```

In the dataset, `raw/crossEndPoint/AndIodine-<TYPE>/*.pcap` are subsets of
`raw/crossEndPoint/AndIodine-<TYPE>.pcap` so they are ignored.
There are other subset pcaps ignored for the same reason.

Merge the split pcaps by running

```bash
./merge-traffic.sh
```

## Convert PCAP to DNS Query Logs

First translate the pcaps into DNS query logs by

```bash
./translate.rb
```

## Preprocessing

Then preprocess the dataset into parquet files by

```bash
./preprocess.py
```

This requires the benign background traffic dataset to be available at `../background/20250716-1020-26h-benign.parquet`.
Unfortunately, due to the sensitive nature of DNS data, we could not public release
the benign background traffic dataset.

If you decide to evaluate with a custom benign background traffic dataset,
you may need to edit `transform_desc.py` in order to make the date time range of
the benign and mailcious traffic overlap.

## From Parquet to Zstandard Compressed CSV

Due to throughput issues, we convert the column oriented parquet to line oriented zstd compressed CSV.

At top level directory, run

```bash
cargo run --release --bin unparquet -- datasets/graph-tunnel/train.parquet
cargo run --release --bin unparquet -- datasets/graph-tunnel/peacetime.parquet
cargo run --release --bin unparquet -- datasets/graph-tunnel/eval.parquet
```

## Metadata Generation

To generate the metadata of this dataset, run the following commands:

```bash
# Generate client list of eval set
./scrape-unique.py eval.parquet client | tee all_clients
# Generate all domains 
env -C ../.. cargo run --release --bin unique-domains  -- datasets/graph-tunnel/eval.parquet  > val_all_domains.list
```