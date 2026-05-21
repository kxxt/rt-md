#!/usr/bin/env python
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import matplotlib.dates as mdates
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "scripts"))

from constants import tz

plt.rcParams["font.family"] = "serif"
plt.rcParams["font.serif"] = ["Times New Roman"] + plt.rcParams["font.serif"]
plt.rcParams["timezone"] = tz

df = pd.read_csv("exports/real-world-resource-usage.csv", parse_dates=["Date"])
throughput = pd.read_csv("exports/real-world-throughput.csv", parse_dates=["Date"])
# df_mva = df.rolling(100).mean()
# print(df_mva)
df["CPU_10"] = df["CPU"].rolling(window=10).mean() * 100
# df["MEM_10"] = df["MEM"].rolling(window=10).mean() * 100
df["MEM"] = df["MEM"] * 100
df_new = df[["Date", "CPU_10", "MEM"]]
df_new = df_new.rename(columns={"CPU_10": "CPU Usage", "MEM": "Memory Usage"})
df_melt = df_new.melt(id_vars=["Date"], var_name="Series", value_name="Value")

fig, ax1 = plt.subplots(figsize=(4, 2.5))
p1 = sns.lineplot(data=df_melt, x="Date", y="Value", hue="Series", ax=ax1)
ax1.set_ylabel("Resource Usage (%)")
ax2 = ax1.twinx()
ax2.set_ylabel("Throughput (QPS)")
p2 = sns.lineplot(
    x="Date",
    y="Value",
    hue="Series",
    data=throughput,
    ax=ax2,
    # legend=False,
    palette=["#ff0000"],
)

h1, l1 = p1.get_legend_handles_labels()
h2, l2 = p2.get_legend_handles_labels()
ax1.legend(handles=h1 + h2, labels=l1 + l2)
ax1.xaxis.set_major_formatter(
    mdates.ConciseDateFormatter(ax1.xaxis.get_major_locator())
)
ax2.get_legend().remove()
sns.move_legend(
    ax1,
    "lower center",
    bbox_to_anchor=(0.5, 1),
    ncol=3,
    title=None,
    frameon=False,
    columnspacing=0.8,
    handlelength=0.7,
)
plt.subplots_adjust(right=1, left=0)
plt.savefig(
    "plot/real-world-throughput-and-resource.pdf", format="pdf", bbox_inches="tight"
)
plt.show()
