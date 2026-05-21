#!/usr/bin/env -S uv run

import gzip
import io
import os
import pyarrow as pa
import pyarrow.parquet as pq
from more_itertools import peekable
from rich.progress import Progress
from datetime import datetime, timedelta

schema = pa.schema(
    [
        pa.field("datetime", pa.timestamp("ms")),
        pa.field("resolver", pa.string()),
        pa.field("client", pa.string()),
        pa.field("query_name", pa.string()),
        pa.field("query_type", pa.string()),
    ]
)
BATCH_SIZE = 5000


def gzip_log_open(file):
    return io.TextIOWrapper(
        gzip.GzipFile(fileobj=file), encoding="utf-8", line_buffering=True
    )


# Iter over logs from a specific resolver, sorted by date
def resolver_log_iter(progress, rootdir, resolver_ip):
    dir = f"{rootdir}/{resolver_ip}"
    for basename in progress.track(
        sorted(os.listdir(dir)), description=f"Log files from {resolver_ip}"
    ):
        path = f"{dir}/{basename}"
        task = progress.add_task(f"Processing {path}")
        _, _, yyyymmdd, hhmm, _, _ = basename.split(".")
        # For dns.last10.20250717.0000.log.gz,
        # it contains some entries from last day,
        # which we should correct by offsetting the day by one
        with (
            progress.open(path, "rb", task_id=task) as f,
            gzip_log_open(f) as g,
        ):
            while line := g.readline():
                fields = line.split()
                if len(fields) < 4:
                    print(f"Warning: invalid log entry from {path}: {line.strip()}, ignoring")
                hhmmssdotms = fields[0]
                src_ip = fields[1]
                query_name = fields[-2]
                qtype = fields[-1]
                date = datetime.fromisoformat(f"{yyyymmdd} {hhmmssdotms}")
                if ".." in query_name or "\\" in query_name:
                    # Skip invalid queries containing empty label or escape sequence
                    # They originally contains invalid characters but the
                    # log preprocessor drops them, leaving an empty label behind
                    continue
                # Skip invalid data
                if len(query_name) > 253:
                    continue
                if hhmm in ("0000", "0004") and date.hour >= 23:
                    date -= timedelta(days=1)
                yield date, resolver_ip, src_ip, query_name, qtype
        progress.remove_task(task)


def min_record(records):
    return min((v for v in records if v is not None), default=None)


def merge(
    progress,
    iter_map,
):
    dates = []
    resolver_ips = []
    src_ips = []
    query_names = []
    qtypes = []
    with (
        pa.OSFile("merged.parquet", "wb") as sink,
        pq.ParquetWriter(
            sink, schema=schema, compression="zstd", compression_level=9
        ) as writer,
    ):
        while True:
            records = (v.peek(None) for v in iter_map.values())
            m = min_record(records)
            if m is None:
                break
            date, resolver_ip, src_ip, query_name, qtype = m
            # Consume the record
            next(iter_map[resolver_ip])

            # Local records
            if src_ip.startswith("127.") or src_ip.startswith("fe80:"):
                continue

            dates.append(date)
            resolver_ips.append(resolver_ip)
            src_ips.append(src_ip)
            query_names.append(query_name)
            qtypes.append(qtype)

            if len(dates) >= BATCH_SIZE:
                writer.write_batch(
                    pa.record_batch(
                        [dates, resolver_ips, src_ips, query_names, qtypes],
                        schema=schema,
                    )
                )
                for buf in dates, src_ips, query_names, qtypes, resolver_ips:
                    buf.clear()


def compute_resolver_iter_map(progress, rootdir):
    resolvers = os.listdir(rootdir)
    return {
        resolver_ip: peekable(resolver_log_iter(progress, rootdir, resolver_ip))
        for resolver_ip in resolvers
    }


if __name__ == "__main__":
    rootdir = "/workspace/dnslog/26h/"
    with Progress() as progress:
        iter_map = compute_resolver_iter_map(progress, rootdir)
        merge(progress, iter_map)
