#!/usr/bin/env -S uv run

import gzip
import io
import os
import multiprocessing
import pyarrow as pa
import pyarrow.parquet as pq
from rich.progress import Progress
from datetime import datetime, timedelta

schema = pa.schema(
    [
        pa.field("datetime", pa.date32()),
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


# Per resolver log agg
def transform_logs(progress, rootdir, resolver_ip):
    dir = f"{rootdir}/{resolver_ip}"
    with (
        pa.OSFile(f"per-resolver/{resolver_ip}.parquet", "wb") as sink,
        pq.ParquetWriter(
            sink, schema=schema, compression="zstd", compression_level=9
        ) as writer,
    ):
        resolver_ips = [resolver_ip] * BATCH_SIZE
        src_ip = []
        query_name = []
        qtype = []
        date = []
        for basename in progress.track(
            sorted(os.listdir(dir)), description=f"Merge data from {resolver_ip}"
        ):
            path = f"{dir}/{basename}"
            task = progress.add_task(f"Processing {path}")
            _, _, yyyymmdd, hhmm, _, _ = basename.split(".")
            with (
                progress.open(path, "rb", task_id=task) as f,
                gzip_log_open(f) as g,
            ):
                while line := g.readline():
                    fields = line.split()
                    hhmmssdotms = fields[0]
                    src_ip.append(fields[1])
                    query_name.append(fields[-2])
                    qtype.append(fields[-1])
                    date.append(datetime.fromisoformat(f"{yyyymmdd} {hhmmssdotms}"))
                    if hhmm in ("0000", "0004") and date.hour >= 23:
                        date -= timedelta(days=1)
                    if len(src_ip) >= 5000:
                        writer.write_batch(
                            pa.record_batch(
                                [date, resolver_ips, src_ip, query_name, qtype],
                                schema=schema,
                            )
                        )
                        for buf in date, src_ip, query_name, qtype:
                            buf.clear()
            progress.remove_task(task)


if __name__ == "__main__":
    rootdir = "/workspace/dnslog/dnslog/"
    resolvers = os.listdir(rootdir)
    os.makedirs("per-resolver", exist_ok=True)
    with Progress() as progress, multiprocessing.Pool() as pool:
        for resolver in progress.track(
            resolvers, description="Create parquet for each resolver"
        ):
            transform_logs(progress, rootdir, resolver)
