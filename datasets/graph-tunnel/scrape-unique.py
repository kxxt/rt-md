#!/usr/bin/env -S uv run

# Scrape unique values of a column from a dataset in parquet format

import pyarrow.parquet as pq
import sys

from multiprocessing import Manager
from concurrent.futures import ProcessPoolExecutor, wait, FIRST_COMPLETED
from functools import reduce
from rich.progress import Progress

CONCURRENCY_LIMIT = 8

def uniques_in_batch(batch):
    # Convert to python str to avoid keeping the reference into parquet alive.
    return {str(v) for v in batch.column(0)}


def unique_values(path_to_parquet, column):
    parquet = pq.ParquetFile(path_to_parquet)
    with (
        Manager() as manager,
        # Progress() as progress,
        ProcessPoolExecutor(max_workers=8) as executor,
    ):
        futures = []
        _progress = manager.dict()
        for i, batch in enumerate(
            parquet.iter_batches(batch_size=500_000, columns=[column])
        ):
            future = executor.submit(uniques_in_batch, batch)
            futures.append(future)
            # Manually rate limiting to avoid OOM
            if sum(fut.running() for fut in futures) > CONCURRENCY_LIMIT:
                wait(futures, return_when=FIRST_COMPLETED)
        wait(futures)
        unique = reduce(lambda x, y: x | y, (fut.result() for fut in futures), set())
    return unique


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <PATH_TO_PARQUET> <COLUMN_NAME>")
        exit(1)

    path_to_parquet = sys.argv[1]
    column_name = sys.argv[2]
    values = unique_values(path_to_parquet, column_name)
    for value in values:
        print(value)
