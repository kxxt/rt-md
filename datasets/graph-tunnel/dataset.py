DATASET_ROOT = "raw"
CROSSENDPOINT_PCAPS = [
    "crossEndPoint/AndIodine-CNAME.pcap",
    "crossEndPoint/AndIodine-MX.pcap",
    "crossEndPoint/AndIodine-NULL.pcap",
    "crossEndPoint/AndIodine-SRV.pcap",
    "crossEndPoint/AndIodine-TXT.pcap",
]
NORMAL_PCAPS = ["normal/normal.pcap"]
TUNNEL_PCAPS = [
    "tunnel/DNS-shell.pcap",
    "tunnel/dnscat2-cname.pcap",
    "tunnel/dnscat2-mx.pcap",
    "tunnel/dnscat2-txt.pcap",
    "tunnel/dnspot.pcap",
    "tunnel/iodine-NULL.pcap",
    "tunnel/iodine-a.pcap",
    "tunnel/iodine-cname.pcap",
    "tunnel/iodine-mx.pcap",
    "tunnel/iodine-private.pcap",
    "tunnel/iodine-srv.pcap",
    "tunnel/iodine-txt.pcap",
    "tunnel/tuns.pcap",
    "tunnel/dns2tcp-key.pcap",
]
UNKNOWN_TUNNEL_PCAPS = [
    "unkownTunnel/cobalstrike.pcap",
    "unkownTunnel/dns2tcp-key.pcap",
    "unkownTunnel/dns2tcp-txt.pcap",
    "unkownTunnel/ozymandns.pcap",
    "unkownTunnel/tcp-over-dns-CNAME.pcap",
    "unkownTunnel/tcp-over-dns-TXT.pcap",
]
WILDCARD_PCAPS = ["wildcard.pcap"]
ALL_PCAPS = (
    CROSSENDPOINT_PCAPS
    + NORMAL_PCAPS
    + TUNNEL_PCAPS
    + UNKNOWN_TUNNEL_PCAPS
    + WILDCARD_PCAPS
)
