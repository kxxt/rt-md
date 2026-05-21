#!/usr/bin/env python

# Generates the unique clients and domains list

from tldextract import TLDExtract

with open("wt_dataset.csv") as f:
    lines = iter(f.readlines())
    next(lines)
    unique_clients = set()
    unique_domains = set()
    extractor = TLDExtract()
    for line in lines:
        ts, domain, client, _ = line.split(",")
        parsed = extractor(domain)
        domain = parsed.registered_domain.lower()
        unique_clients.add(client)
        unique_domains.add(domain)
    
with open("unique_clients.txt", "w") as f:
    for client in unique_clients:
        f.write(f"{client}\n")

with open("unique_domains.txt", "w") as f:
    for domain in unique_domains:
        f.write(f"{domain}\n")
