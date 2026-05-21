#!/usr/bin/env -S uv run

from base64 import urlsafe_b64encode
from datetime import datetime, timedelta
from math import floor, ceil
from more_itertools import peekable


def min_record(records):
    return min((v for v in records if v is not None), default=None)


# A generator for adversarial multiple domain exfiltration

def exfiltration(data: str, ip, start_date, data_rate, domains, deltas):
    """
    Generates one exfiltration attempt.
    """
    assert len(domains) == len(deltas)
    # url-safe base64 encode data
    b64data = urlsafe_b64encode(data.encode('utf-8')).decode('utf-8')
    # Remove padding '=' characters
    b64data = b64data.strip('=')

    # partition the data according to domains
    bytes_per_part = floor(len(b64data) / len(domains))
    bytes_last_part = len(b64data) - bytes_per_part * (len(domains) - 1)
    data_rate = data_rate / len(domains)
    partition = [bytes_per_part] * (len(domains) - 1) + [bytes_last_part]
    partitioned_data = []
    acc = 0
    
    for part in partition:
        partitioned_data.append(b64data[acc : acc + part])
        acc += part
    iters = {
        domain: peekable(
            single_domain_exfiltration(
                partitioned_data[i], start_date + delta, data_rate, domain
            )
        )
        for i, (domain, delta) in enumerate(zip(domains, deltas))
    }
    while True:
        records = (v.peek(None) for v in iters.values())
        m = min_record(records)
        if m is None:
            break
        ts, req, domain = m
        next(iters[domain])
        yield ts, ip, req


def single_domain_exfiltration(data: str, start_date, data_rate, domain):
    """
    Perform a single domain exfiltration
    (as part of multiple-domain exfiltration)
    """

    # Accumulate data until we reach a reasonable size,
    # otherwise we risk using non unique subdomains that could be cached

    min_len = 32
    delta_t = timedelta(seconds=ceil(min_len / data_rate))
    n = ceil(len(data) / min_len)

    for i in range(n):
        end = min(len(data), (i + 1) * min_len)
        ts = start_date + delta_t * i
        yield ts, f"{data[i * min_len : end]}.{domain}", domain
