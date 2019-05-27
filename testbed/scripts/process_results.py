import json
import sys
from functools import reduce

nodes_file = sys.argv[1]
duration = int(sys.argv[2])
nodes = []

with open(nodes_file) as f:
    for line in f:
        i = line.rstrip().split(",")
        nodes.append(i)

tx_gen = {};
tx_gen_bytes = {};
tx_gen_fails = 0;
tx_confirmed = {};
tx_confirmed_bytes = {};

for node in nodes:
    name = node[0]
    with open("data/{}_get_performance.txt".format(name)) as f:
        perf = json.load(f)
        tx_gen[name] = perf['generated_transactions']
        tx_gen_bytes[name] = perf['generated_transaction_bytes']
        tx_gen_fails += perf['generate_transaction_failures']
        tx_confirmed[name] = perf['confirmed_transactions']
        tx_confirmed_bytes[name] = perf['confirmed_transaction_bytes']

print("Transaction generation")
for node in nodes:
    name = node[0]
    print("{:.2f} Tx/s\t({:.2f} B/s) at {}".format(tx_gen[name] / duration, tx_gen_bytes[name]/duration, name))
print("{:.2f} Tx/s\t({:.2f} B/s) total".format(reduce((lambda x, y: x + y), tx_gen.values()) / duration, reduce((lambda x, y: x + y), tx_gen_bytes.values()) / duration))

print("Transaction confirmation")
for node in nodes:
    name = node[0]
    print("{:.2f} Tx/s\t({:.2f} B/s) at {}".format(tx_confirmed[name] / duration, tx_confirmed_bytes[name] / duration, name))
print("{:.2f} Tx/s\t({:.2f} B/s) average".format(reduce((lambda x, y: x + y), tx_confirmed.values()) / duration / len(nodes), reduce((lambda x, y: x+ y), tx_confirmed_bytes.values()) / duration / len(nodes), name))

print("Transaction generation failures: {:.2f} Txs".format(tx_gen_fails))
