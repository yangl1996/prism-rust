#!/usr/local/bin/python3

import sys
import json 

if len(sys.argv) != 2:
    print("Usage: ./fix_json.py <file to fix>")
    exit(1)

filepath = sys.argv[1]
print("Opening " + filepath)

with open(filepath) as f:
    dt = json.load(f)

miner = dt['miner']
print(miner)

btc_conn = []
for node in dt['nodes']:
    if node['name'] == miner:
        continue
    btc_conn.append({'src': node['name'], 'dst': miner})
dt['btcd_connections'] = btc_conn

for ch in dt['lnd_channels']:
    if ch['capacity'] == 1000000000:
        ch['capacity'] = 100000000
    elif ch['capacity'] == 50:
        ch['capacity'] = 10000
    else:
        print("Invalid channel capacity")

for dm in dt['demands']:
    dm['rate'] = int(dm['rate'])

with open(filepath, 'w') as f:
    json.dump(dt, f, indent=4)
