#!/usr/bin/env python3

# Read human ops from RT-MD output

import sys
import datetime
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import numpy as npy
import matplotlib.dates as mdates

from constants import exp_end, exp_start, tz

plt.rcParams["font.family"] = "serif"
plt.rcParams["font.serif"] = ["Times New Roman"] + plt.rcParams["font.serif"]
plt.rcParams['timezone'] = tz

allowlisted = 0
dismissed = 0
ignored = 0


df = pd.DataFrame({"Action": [], "Value": [], "Date": []})


def main():
    global allowlisted, dismissed, ignored
    with open(sys.argv[1]) as f:
        lines = f.readlines()
    for s in ["Allowlist", "Ignore Host", "Dismiss Alert"]:
        df.loc[len(df)] = {
            "Value": 0,
            "Action": s,
            "Date": pd.Timestamp(exp_start, unit="s", tz=tz),
        }
    for line in lines:
        if not line.startswith("["):
            # Alerts
            continue
        [date, _] = line.split("]", maxsplit=1)
        date = pd.Timestamp(int(date[1:]), unit="ms", tz=tz)

        if "Allowlisted" in line:
            # [1765243476932] Allowlisted cbnodes.toptmux
            allowlisted += 1
            df.loc[len(df)] = {
                "Value": allowlisted,
                "Action": "Allowlist",
                "Date": date,
            }
        elif "Ignored host" in line:
            # [1765242725663] Ignored host 202.38.64.8
            ignored += 1
            df.loc[len(df)] = {
                "Value": ignored,
                "Action": "Ignore Host",
                "Date": date,
            }
        elif "Dismissed alert from host" in line:
            # [1765265849747] Dismissed alert from host 222.195.93.112
            dismissed += 1
            df.loc[len(df)] = {
                "Value": dismissed,
                "Action": "Dismiss Alert",
                "Date": date,
            }
        else:
            continue

    for s in ["Allowlist", "Ignore Host", "Dismiss Alert"]:
        df.loc[len(df)] = {
            "Value": {
                "Allowlist": allowlisted,
                "Ignore Host": ignored,
                "Dismiss Alert": dismissed,
            }[s],
            "Action": s,
            "Date": pd.Timestamp(exp_end, unit="s", tz=tz),
        }

    # sns.set_style("whitegrid")
    df.to_csv("exports/real-world-human-ops.csv")
    plt.figure(figsize=(4, 2))
    p = sns.lineplot(data=df, x="Date", y="Value", hue="Action", ) # marker="o"
    # plt.xticks(rotation=45)
    p.xaxis.set_major_formatter(mdates.ConciseDateFormatter(p.xaxis.get_major_locator()))

    # p.set(yticks=npy.arange(0, 21, 2))
    sns.move_legend(
        p,
        "lower center",
        bbox_to_anchor=(0.5, 1),
        ncol=3,
        title=None,
        frameon=False,
        columnspacing=0.8,
    )
    plt.subplots_adjust(right=1)
    # sns.scatterplot(data=df, x="Date", y="Total", hue="IP", s=5, edgecolor="none")
    # plt.title("Cumulative Human Operations Over Time")
    plt.savefig("plot/real-world-human-ops.pdf", format="pdf", bbox_inches="tight")
    plt.show()


if __name__ == "__main__":
    main()
