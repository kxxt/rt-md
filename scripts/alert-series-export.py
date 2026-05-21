#!/usr/bin/env python3

# Read alert series

import sys
import matplotlib.pyplot as plt
import seaborn as sns
import pandas as pd
import json
import numpy as npy
import matplotlib.dates as mdates

from constants import tp_hosts, tz

plt.rcParams["font.family"] = "serif"
plt.rcParams["font.serif"] = ["Times New Roman"] + plt.rcParams["font.serif"]
plt.rcParams['timezone'] = tz

df = pd.DataFrame({"Count": [], "Series": [], "Date": []})

def main():
    cum_tp = 0
    cum_fp = 0
    cum_eliminated_fp = 0
    last_tp = set()
    last_fp = set()
    with open(sys.argv[1]) as f:
        lines = f.readlines()
    for line in lines:
        j = json.loads(line)
        ts = pd.Timestamp(j["timestamp"], unit="s", tz=tz)
        alerts = j.keys() - {"timestamp"}
        tp = alerts & tp_hosts
        fp = alerts - tp_hosts
        cum_tp += len(tp - last_tp)
        cum_fp += len(fp - last_fp)
        cum_eliminated_fp += len(last_fp - fp)
        # n_tp = len(tp)
        # n_fp = len(fp)
        last_fp = fp
        last_tp = tp
        df.loc[len(df)] = {"Date": ts, "Count": cum_tp, "Series": "TP"}
        df.loc[len(df)] = {"Date": ts, "Count": cum_fp, "Series": "FP"}
        df.loc[len(df)] = {"Date": ts, "Count": cum_eliminated_fp, "Series": "Eliminated FP"}
        df.loc[len(df)] = {"Date": ts, "Count": len(fp), "Series": "Uninvestigated"}

    df.to_csv("exports/real-world-alert-series.csv")
    plt.figure(figsize=(4, 2))
    p = sns.lineplot(data=df, x="Date", y="Count", hue="Series")
    # p.set(yticks=npy.arange(0, 5, 1))
    sns.move_legend(
        p,
        "lower center",
        bbox_to_anchor=(0.5, 1),
        ncol=4,
        title=None,
        frameon=False,
        columnspacing=0.8,
        handlelength=0.7
    )
    plt.subplots_adjust(right=1)
    p.xaxis.set_major_formatter(mdates.ConciseDateFormatter(p.xaxis.get_major_locator()))
    # plt.title("Remaining Alerts Over Time")
    plt.savefig("plot/real-world-alerts.pdf", format="pdf", bbox_inches="tight")
    plt.show()


if __name__ == "__main__":
    main()
