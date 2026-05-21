#!/usr/bin/env python3

# Read alerts from RT-MD output and summarize them according to hosts

import sys
import os
import json
import datetime
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd

from constants import tz

hosts = {}

plt.rcParams["font.family"] = "serif"
plt.rcParams["font.serif"] = ["Times New Roman"] + plt.rcParams["font.serif"]
plt.rcParams['timezone'] = tz

df = pd.DataFrame({"IP": [], "Total": [], "Date": []})

def main():
    with open(sys.argv[1]) as f:
        lines = f.readlines()
    for line in lines:
        if not ("Client" in line and "suspicious" in line):
            continue
        if line.startswith("["):
            # Deducted or Dismissed
            continue
        if line.startswith("Alert! "):
            line = line.removeprefix("Alert! ")
        if line.startswith("Throttled alert! "):
            line = line.removeprefix("Throttled alert! ")
        # Parse date and total
        # Tue Dec  2 06:25:55 2025 Client XXX is suspicious, total=1240, top_domains=P
        [dates, rest] = line.split(" Client ", maxsplit=1)
        date = datetime.datetime.strptime(dates, "%c")
        [ip, rest] = rest.split(" is suspicious, total=", maxsplit=1)
        [total, rest] = rest.split(",", maxsplit=1)
        total = int(total)
        hosts[ip] = (date, total)
        df.loc[len(df)] = {"IP": ip, "Total": total, "Date": pd.to_datetime(date)}

    df.to_csv("exports/real-world-alerts.csv")
    plt.figure(figsize=(100, 6))
    sns.scatterplot(data=df, x="Date", y="Total", hue="IP", s=5, edgecolor="none")
    plt.title("Alerts over time")
    plt.savefig("plot/real-world-alerts.pdf", format="pdf", bbox_inches="tight")
    plt.show()


if __name__ == "__main__":
    main()
