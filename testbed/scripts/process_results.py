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

tx_gen_fails = 0;
data = {}

for node in nodes:
    name = node[0]
    with open("data/{}_get_performance.txt".format(name)) as f:
        perf = json.load(f)
        data[name] = perf
        tx_gen_fails += perf['generate_transaction_failures']

def print_node_results(display_name, metrics, reduction):
    print(display_name)
    for node in nodes:
        name = node[0]
        result_string = '{}: '.format(name)
        results = []
        for m in metrics:
            field = m[0]
            unit = m[1]
            result = "{:.2f} {}/s".format(data[name][field] / duration, unit)
            results.append(result)
        result_string += '\t'.join(results)
        print(result_string)

    results = []
    for m in metrics:
        field = m[0]
        unit = m[1]
        if reduction=='sum':
            reduced = 0
            for node in nodes:
                name = node[0]
                reduced += data[name][field]
            result = "{:.2f} {}/s".format(reduced / duration, unit)
            results.append(result)
            result_string = "Total: "
        elif reduction=='average':
            reduced = 0
            for node in nodes:
                name = node[0]
                reduced += data[name][field]
            result = "{:.2f} {}/s".format(reduced / duration / len(nodes), unit)
            results.append(result)
            result_string = "Average: "
    result_string += '\t'.join(results)
    print(result_string)

print_node_results("Transaction generation", [('generated_transactions', 'Txs'), ('generated_transaction_bytes', 'B')], 'sum')
print_node_results("Transaction Block confirmation", [('confirmed_transaction_blocks', 'Blks')], 'average')
print_node_results("Transaction confirmation", [('confirmed_transactions', 'Txs'), ('confirmed_transaction_bytes', 'B')], 'average')
print_node_results("Block processed (proposer, voter, transaction)", [('processed_proposer_blocks', 'Blks'), ('processed_voter_blocks', 'Blks'), ('processed_transaction_blocks', 'Blks')], 'average')
print_node_results("Block mined (proposer, voter, transaction)", [('mined_proposer_blocks', 'Blks'), ('mined_voter_blocks', 'Blks'), ('mined_transaction_blocks', 'Blks')], 'average')

print("Transaction generation failures: {:.2f} Txs".format(tx_gen_fails))


