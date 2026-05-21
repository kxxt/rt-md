#!/usr/bin/env -S uv run
from dataset import DATASET_ROOT, ALL_PCAPS
from dpkt.pcap import UniversalReader
from rich.progress import Progress
import matplotlib.pyplot as plt
from datetime import datetime, timedelta, UTC


def get_pcap_timestamp(f):
    packets = UniversalReader(f)
    first, _ = next(packets)
    *_, (last, _) = packets
    return (
        datetime.fromtimestamp(first, UTC),
        datetime.fromtimestamp(last, UTC),
    )


if __name__ == "__main__":
    print("Plotting dataset")
    timestamps = {}
    with Progress() as progress:
        for path in progress.track(ALL_PCAPS, description="Processing pcaps"):
            task = progress.add_task(f"Reading {path}")
            full_path = f"{DATASET_ROOT}/{path}"
            with progress.open(full_path, "rb", task_id=task) as f:
                timestamps[path] = get_pcap_timestamp(f)
                print(
                    f"Duration of {path}: {timestamps[path][1] - timestamps[path][0]}"
                )
            progress.remove_task(task)
        plt.figure(figsize=(20, 6))
        malicious_total_duration = sum(
            (
                end - start
                for path, (start, end) in timestamps.items()
                if path != "normal/normal.pcap" and path != "wildcard.pcap"
            ),
            timedelta(0),
        )
        plt.hlines(
            list(timestamps.keys()) + ["malicious"],
            [v[0] for v in timestamps.values()] + [timestamps["normal/normal.pcap"][0]],
            [v[1] for v in timestamps.values()]
            + [timestamps["normal/normal.pcap"][0] + malicious_total_duration],
            [f"C{i}" for i in range(len(timestamps))] + ["C100"],
        )
        print(f"Malicious duration: {malicious_total_duration}")
        print(
            f"Normal duration: {timestamps['normal/normal.pcap'][1] - timestamps['normal/normal.pcap'][0]}"
        )
        print(
            f"Wildcard duration: {timestamps['wildcard.pcap'][1] - timestamps['wildcard.pcap'][0]}"
        )
        plt.savefig("range.png")
