#!/usr/bin/env python3

# Read human ops from RT-MD output

import sys
import pandas as pd

from constants import tz

df = pd.DataFrame({"Series": [], "Value": [], "Date": []})


def main():
    global allowlisted, dismissed, ignored
    with open(sys.argv[1]) as f:
        lines = f.readlines()
    for line in lines:
        if not line.startswith("["):
            # Alerts
            continue
        [date, rest] = line.split("]", maxsplit=1)
        date = pd.Timestamp(int(date[1:]), unit="ms", tz=tz)
        if rest.strip().startswith("Throughput: "):
            # [1764656218211] Throughput: 14940.316666666668
            throughput = float(rest.strip().split()[1])
            df.loc[len(df)] = {
                "Value": throughput,
                "Series": "Throughput",
                "Date": date,
            }

    df.to_csv("exports/real-world-throughput.csv")


if __name__ == "__main__":
    main()
