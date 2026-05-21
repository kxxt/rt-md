#!/usr/bin/env -S uv run
from transform_desc import SUBDATASET_GROUPS
from dataset import DATASET_ROOT
from rich.progress import Progress
from datetime import datetime
from more_itertools import peekable


def subdataset_iter(progress, subdataset):
    """
    An iter over a defined SubDataset
    """
    raw_iter = peekable(subdataset_raw_iter(progress, subdataset))
    dt, *_ = raw_iter.peek()
    offset = subdataset.start_time + subdataset.offset - dt
    for dt, client, server, qname, qtype in raw_iter:
        yield dt + offset, client, server, qname, qtype


def subdataset_raw_iter(progress, subdataset):
    # Open a subdataset according to its desc
    task = progress.add_task(f"Reading {subdataset.path}.log")
    with progress.open(f"{DATASET_ROOT}/{subdataset.path}.log", "r", task_id=task) as f:
        while line := f.readline():
            (
                _id,
                ts,
                client,
                _,
                server,
                _,
                _,
                _,
                _,
                _trans_id,
                qtype,
                qname,
                *_rest,
            ) = line.strip().split()
            client = subdataset.nat.get(client, client)
            for orig, target in subdataset.dnt.items():
                hmm = qname.removesuffix("." + orig)
                if hmm != qname:
                    qname = hmm + "." + target
            yield (datetime.fromtimestamp(float(ts)), client, server, qname, qtype)
    progress.remove_task(task)

def obtain_iters_for_group(progress, group):
    return [peekable(subdataset_iter(progress, subdataset)) for subdataset in group]


def min_record(records):
    return min((v for v in records if v[0] is not None), default=(None, None))


def transform_iter(progress):
    """
    Iterate through the DNS queries in transformed SubDataset Groups
    """
    group_progress = progress.add_task("Iter GraphTunnel malicious dataset groups")
    for group in progress.track(SUBDATASET_GROUPS, task_id=group_progress):
        # iter over all datasets in this subgroup
        iters = obtain_iters_for_group(progress, group)
        while True:
            records = ((v.peek(None), index) for index, v in enumerate(iters))
            m, index = min_record(records)
            if m is None:
                break
            next(iters[index])
            yield m


if __name__ == "__main__":
    with Progress() as progress:
        for dt, client, server, qname, qtype in transform_iter(progress):
            print(dt, client, server, qname, qtype)
