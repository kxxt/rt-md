#!/usr/bin/env -S uv run
from dataset import DATASET_ROOT, ALL_PCAPS
from dpkt.pcap import UniversalReader
from rich.progress import Progress
from datetime import datetime, timedelta, UTC
from more_itertools import peekable
import pyarrow as pa
import pyarrow.parquet as pq
from os import environ

from transform import transform_iter

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

TRAIN_TIME = timedelta(hours=10)
PEACE_TIME = timedelta(hours=2)
BACKGROUND_DNS_LOG = "../background/background.parquet"
HAS_BACKGROUND_DATASET = not bool(environ['NO_BACKGROUND_DATASET'])

# Preprocess the dataset
#    1. Rewrite malicious pcap to use unique source IPs
#    2. Rewrite malicious pcap to use unique primary domains
#    4. Rewrite timestamps of malicious traffic
#    5. Merge all pcaps


def parquet_iter(progress, path):
    with progress.open(path, mode="rb", description="Reading benign logs") as f:
        parquet = pq.ParquetFile(f)
        for i, batch in enumerate(parquet.iter_batches(batch_size=500_000)):
            dt = batch.column("datetime")
            resolver = batch.column("resolver")
            client = batch.column("client")
            qname = batch.column("query_name")
            qtype = batch.column("query_type")
            total = len(dt)
            for i in range(total):
                yield dt[i], resolver[i], client[i], qname[i], qtype[i]


if __name__ == "__main__":
    print("Preprocessing dataset")
    timestamps = {}
    with Progress() as progress:
        benign = peekable(parquet_iter(progress, BACKGROUND_DNS_LOG))
        dates = []
        resolver_ips = []
        src_ips = []
        query_names = []
        qtypes = []
        benign_start_dt, *_ = benign.peek()
        benign_start_dt = benign_start_dt.as_py()
        train_end_dt = benign_start_dt + TRAIN_TIME
        peace_end_dt = benign_start_dt + TRAIN_TIME + PEACE_TIME
        # Train
        print("Writing training set")
        with (
            pa.OSFile("train.parquet", "wb") as sink,
            pq.ParquetWriter(
                sink, schema=schema, compression="zstd", compression_level=9
            ) as writer,
        ):
            while HAS_BACKGROUND_DATASET:
                dt, resolver, client, qname, qtype = benign.peek()
                if dt.as_py() > train_end_dt:
                    break
                next(benign)
                dates.append(dt)
                resolver_ips.append(resolver)
                src_ips.append(client)
                query_names.append(qname)
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
            writer.write_batch(
                pa.record_batch(
                    [dates, resolver_ips, src_ips, query_names, qtypes],
                    schema=schema,
                )
            )
            for buf in dates, src_ips, query_names, qtypes, resolver_ips:
                buf.clear()
        # Peacetime
        print("Writing peacetime set")
        with (
            pa.OSFile("peacetime.parquet", "wb") as sink,
            pq.ParquetWriter(
                sink, schema=schema, compression="zstd", compression_level=9
            ) as writer,
        ):
            while HAS_BACKGROUND_DATASET:
                dt, resolver, client, qname, qtype = benign.peek()
                if dt.as_py() > peace_end_dt:
                    break
                next(benign)
                dates.append(dt)
                resolver_ips.append(resolver)
                src_ips.append(client)
                query_names.append(qname)
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
            writer.write_batch(
                pa.record_batch(
                    [dates, resolver_ips, src_ips, query_names, qtypes],
                    schema=schema,
                )
            )
            for buf in dates, src_ips, query_names, qtypes, resolver_ips:
                buf.clear()
        # Eval set
        print("Writing eval set")
        malicious = peekable(transform_iter(progress))
        with (
            pa.OSFile("eval.parquet", "wb") as sink,
            pq.ParquetWriter(
                sink, schema=schema, compression="zstd", compression_level=9
            ) as writer,
        ):
            while True:
                b_dt, b_resolver, b_client, b_qname, b_qtype = benign.peek(
                    (None, None, None, None, None)
                )
                m_dt, m_client, m_resolver, m_qname, m_qtype = malicious.peek(
                    (None, None, None, None, None)
                )
                if b_dt is None and m_dt is None:
                    break
                elif b_dt is None:
                    # Write m_dt
                    next(malicious)
                    dates.append(m_dt)
                    resolver_ips.append(m_resolver)
                    src_ips.append(m_client)
                    query_names.append(m_qname)
                    qtypes.append(m_qtype)
                elif m_dt is None:
                    # Write b_dt
                    next(benign)
                    dates.append(b_dt)
                    resolver_ips.append(b_resolver)
                    src_ips.append(b_client)
                    query_names.append(b_qname)
                    qtypes.append(b_qtype)
                elif b_dt.as_py() <= m_dt:
                    # Write b_dt
                    next(benign)
                    dates.append(b_dt)
                    resolver_ips.append(b_resolver)
                    src_ips.append(b_client)
                    query_names.append(b_qname)
                    qtypes.append(b_qtype)
                else:
                    # Write m_dt
                    next(malicious)
                    dates.append(m_dt)
                    resolver_ips.append(m_resolver)
                    src_ips.append(m_client)
                    query_names.append(m_qname)
                    qtypes.append(m_qtype)

                if len(dates) >= BATCH_SIZE:
                    writer.write_batch(
                        pa.record_batch(
                            [dates, resolver_ips, src_ips, query_names, qtypes],
                            schema=schema,
                        )
                    )
                    for buf in dates, src_ips, query_names, qtypes, resolver_ips:
                        buf.clear()
            writer.write_batch(
                pa.record_batch(
                    [dates, resolver_ips, src_ips, query_names, qtypes],
                    schema=schema,
                )
            )
            for buf in dates, src_ips, query_names, qtypes, resolver_ips:
                buf.clear()
        print("Done")
