#!/usr/bin/env python

import ijson
import sys
import datetime
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from zoneinfo import ZoneInfo

from constants import tz

df = pd.DataFrame({"CPU": [], "MEM": [], "Date": []})

f = open(sys.argv[1])
parser = ijson.parse(f)

yy = 0
mm = 0
dd = 0

HH = 0
MM = 0
SS = 0
cpu = 0
mem = 0

tz = ZoneInfo(tz)

for prefix, event, value in parser:
    if prefix == "sysstat.hosts.item.date":
        mm, dd, yy = [int(x) for x in value.split("/", maxsplit=2)]
        yy += 2000
    elif prefix == "sysstat.hosts.item.statistics.item" and event == "end_map":
        # print(event, value)
        date = datetime.datetime(yy, mm, dd, HH, MM, SS, 0, datetime.UTC)
        date = date.astimezone(tz)
        date = pd.to_datetime(date)
        # Send value to dataframe
        df.loc[len(df)] = {
            "CPU": cpu,
            "MEM": mem,
            "Date": date,
        }
    elif prefix == "sysstat.hosts.item.statistics.item.timestamp" and event == "string":
        # Parse timestamp
        h, m, s = [int(x) for x in value.split(":", maxsplit=2)]
        if h < HH:
            # A new day begins
            dd += 1
        HH = h
        MM = m
        SS = s
    elif (
        prefix == "sysstat.hosts.item.statistics.item.task-cpu-load.item.cpu"
        and event == "number"
    ):
        cpu = float(value) * 0.01
    elif (
        prefix == "sysstat.hosts.item.statistics.item.task-memory.item.MEM"
        and event == "number"
    ):
        mem = float(value) * 0.01

df.to_csv("exports/real-world-resource-usage.csv")

# plt.figure(figsize=(10, 6))
# sns.lineplot(data=df, x="Date", y="Value", hue="Series")
# # sns.scatterplot(data=df, x="Date", y="Total", hue="IP", s=5, edgecolor="none")
# plt.title("Resource Usage")
# plt.savefig("plot/real-world-resource-usage.pdf", format="pdf", bbox_inches="tight")
# plt.show()

f.close()
