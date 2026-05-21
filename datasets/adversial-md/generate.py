#!/usr/bin/env -S uv run
from datetime import timedelta, datetime

from generator import exfiltration


def iter(start_date):
    with open("lorem.txt") as f:
        data = f.read()
    # Two domain
    last_date = start_date
    for dat in exfiltration(
        data,
        "192.168.200.1",
        last_date,
        48.5,  # 48.5 units/second is about 32 bytes/s
        ["dns-exf2-0.com", "dns-exf2-1.com"],
        [timedelta(), timedelta(seconds=30)],
    ):
        last_date = dat[0]
        yield dat

    # Four domain
    for dat in exfiltration(
        data,
        "192.168.200.2",
        last_date,
        48.5,
        [f"dns-exf4-{i}.com" for i in range(4)],
        [
            timedelta(),
            timedelta(seconds=30),
            timedelta(seconds=10),
            timedelta(seconds=20),
        ],
    ):
        last_date = dat[0]
        yield dat

    # eight domain
    for dat in exfiltration(
        data,
        "192.168.200.3",
        last_date,
        48.5,
        [f"dns-exf8-{i}.com" for i in range(8)],
        [
            timedelta(),
        ]
        * 8,
    ):
        last_date = dat[0]
        yield dat

    # 16 domain
    for dat in exfiltration(
        data,
        "192.168.200.4",
        last_date,
        48.5,
        [f"dns-exf16-{i}.com" for i in range(16)],
        [
            timedelta(),
        ]
        * 16,
    ):
        last_date = dat[0]
        yield dat

    # 32 domain
    for dat in exfiltration(
        data,
        "192.168.200.5",
        last_date,
        48.5,
        [f"dns-exf32-{i}.com" for i in range(32)],
        [
            timedelta(),
        ]
        * 32,
    ):
        last_date = dat[0]
        yield dat

    # 64 domain
    for dat in exfiltration(
        data,
        "192.168.200.6",
        last_date,
        48.5,
        [f"dns-exf64-{i}.com" for i in range(64)],
        [
            timedelta(),
        ]
        * 64,
    ):
        last_date = dat[0]
        yield dat

    # 2 domain, 1 byte/s
    for dat in exfiltration(
        data,
        "192.168.200.7",
        last_date,
        1.5,  # nearly 1 byte/second
        [f"dns-exfslow-2-{i}.com" for i in range(2)],
        [
            timedelta(),
            timedelta(seconds=10),
        ],
    ):
        last_date = dat[0]
        yield dat

    # 4 domain, 1 byte/s
    for dat in exfiltration(
        data,
        "192.168.200.8",
        last_date,
        1.5,  # nearly 1 byte/second
        [f"dns-exfslow-4-{i}.com" for i in range(4)],
        [
            timedelta(),
            timedelta(seconds=22.5),
            timedelta(seconds=45),
            timedelta(seconds=67.5),
        ],
    ):
        last_date = dat[0]
        yield dat

    # 8 domain, 1 byte/s
    for dat in exfiltration(
        data,
        "192.168.200.9",
        last_date,
        1.5,  # nearly 1 byte/second
        [f"dns-exfslow-8-{i}.com" for i in range(8)],
        [
            timedelta(),
            timedelta(seconds=25),
            timedelta(seconds=50),
            timedelta(seconds=75),
            timedelta(seconds=100),
            timedelta(seconds=125),
            timedelta(seconds=150),
            timedelta(seconds=175),
        ],
    ):
        last_date = dat[0]
        yield dat


if __name__ == "__main__":
    print("datetime, client, qname")
    for ts, ip, req in iter(datetime(2025, 9, 10, 11, 0)):
        print(f"{int(ts.timestamp() * 1000)}, {ip}, {req}")
