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

with (
    pa.OSFile("background.parquet", "wb") as sink,
    pq.ParquetWriter(
        sink, schema=schema, compression="zstd", compression_level=9
    ) as writer,
):
    # The merge script expects a parquet with at least two data.
    dates = [datetime.fromisoformat(f"2025-07-31T14:25:02.473"), datetime.fromisoformat(f"2025-07-31T14:25:04.473")]
    resolver_ips = ["8.8.8.8", "8.8.8.8"]
    src_ips = ["1.1.1.1", "1.1.1.1"]
    query_names = ["example.com", "example.com"]
    qtypes = ["A", "A"]
    writer.write_batch(
        pa.record_batch(
            [dates, resolver_ips, src_ips, query_names, qtypes],
            schema=schema,
        )
    )