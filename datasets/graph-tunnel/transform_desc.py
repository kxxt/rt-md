from dataclasses import dataclass, field
from datetime import timedelta, datetime
from dataset import CROSSENDPOINT_PCAPS


@dataclass
class SubDataset:
    start_time: datetime
    offset: timedelta
    path: str
    scale: float = 1
    # Pseudo Network Address translation
    nat: dict = field(default_factory=dict)
    # Domain Name translation
    dnt: dict = field(default_factory=dict)


class RangeAllocator:
    def __init__(self):
        self.allocated = 0

    def allocate(self, length):
        start = self.allocated
        end = self.allocated + length
        self.allocated = end
        return range(start, end)


# PNAT Range:
#   192.168.100.0/24

# DNT mapping
#   ggy666.tk -> ggy6{counter:02}.tk

PNAT_ALLOCATOR = RangeAllocator()
DNT_ALLOCATOR = RangeAllocator()

START_TIME = datetime(2025, 7, 31, 11, 30)

SUBDATASET_GROUPS = [
    # 11:00, crossEndPoint exfiltration AndIodine
    [
        SubDataset(
            START_TIME,
            timedelta(hours=11),
            path,
            nat={"172.16.1.15": f"192.168.100.{i + 1}"},
            dnt={"ggy666.tk": f"ggy6{i + 1:02}.tk"},
        )
        for i, path in zip(
            PNAT_ALLOCATOR.allocate(len(CROSSENDPOINT_PCAPS)), CROSSENDPOINT_PCAPS
        )
    ],
    # 13:30. DNS-shell, dnscat2-* and dnspot
    [
        SubDataset(
            START_TIME,
            timedelta(hours=13, minutes=30),
            path,
            nat={
                "192.168.68.62": f"192.168.100.{i + 1}",
                "192.168.117.128": f"192.168.100.{i + 1}",
            },
            dnt={"ggy666.tk": f"ggy6{i + 1:02}.tk"},
        )
        for i, path in zip(
            PNAT_ALLOCATOR.allocate(5),
            (
                "tunnel/DNS-shell.pcap",
                "tunnel/dnscat2-cname.pcap",
                "tunnel/dnscat2-mx.pcap",
                "tunnel/dnscat2-txt.pcap",
                "tunnel/dnspot.pcap",
            ),
        )
    ],
    # # 15:45, iodine Note: Removed since iodine uses invalid characters in queries, the resolver would reject such queries in our threat model.
    # [
    #     SubDataset(
    #         START_TIME,
    #         timedelta(hours=15, minutes=45),
    #         path,
    #         nat={
    #             "192.168.117.128": f"192.168.100.{i + 1}",
    #         },
    #         dnt={"ggy666.tk": f"ggy6{i + 1:02}.tk"},
    #     )
    #     for i, path in zip(
    #         PNAT_ALLOCATOR.allocate(6),
    #         (
    #             "tunnel/iodine-NULL.pcap",  # 192.168.117.128
    #             "tunnel/iodine-a.pcap",
    #             # "tunnel/iodine-cname.pcap",
    #             # "tunnel/iodine-mx.pcap",
    #             # "tunnel/iodine-private.pcap",
    #             "tunnel/iodine-srv.pcap",
    #             "tunnel/iodine-txt.pcap",
    #         ),
    #     )
    # ],
    # 17:00
    [
        SubDataset(
            START_TIME,
            timedelta(hours=17, minutes=00),
            path,
            nat={
                "192.168.68.62": f"192.168.100.{i + 1}",
                "192.168.68.114": f"192.168.100.{i + 1}",  # dns2tcp-key
                "192.168.117.128": f"192.168.100.{i + 1}",  # dns2tcp-txt, ozymandns
                "113.54.129.232": f"192.168.100.{i + 1}",  # tcp-over-dns-CNAME, tcp-over-dns-TXT
            },
            dnt={"ggy666.tk": f"ggy6{i + 1:02}.tk"},
        )
        for i, path in zip(
            PNAT_ALLOCATOR.allocate(5),
            (
                # "tunnel/dns2tcp-key.pcap", # duplicate of unkownTunnel/dns2tcp-key.pcap
                "unkownTunnel/dns2tcp-key.pcap",
                "unkownTunnel/dns2tcp-txt.pcap",
                "unkownTunnel/ozymandns.pcap",
                "unkownTunnel/tcp-over-dns-CNAME.pcap",
                "unkownTunnel/tcp-over-dns-TXT.pcap",
            ),
        )
    ],
    # 21:40
    [
        SubDataset(
            START_TIME,
            timedelta(hours=21, minutes=40),
            path,
            nat={
                "192.168.117.128": f"192.168.100.{i + 1}",
                "192.168.68.62": f"192.168.100.{i + 1}",
            },
            dnt={"ggy666.tk": f"ggy6{i + 1:02}.tk"},
        )
        for i, path in zip(
            PNAT_ALLOCATOR.allocate(2),
            (
                "unkownTunnel/cobalstrike.pcap",
                "tunnel/tuns.pcap",
            ),
        )
    ],
]
