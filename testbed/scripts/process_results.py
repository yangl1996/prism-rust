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

def print_node_results(display_name, metrics, reduction, mapping='average'):
    print(display_name)
    for node in nodes:
        name = node[0]
        result_string = '{}: '.format(name)
        results = []
        for m in metrics:
            field = m[0]
            unit = m[1]
            if mapping=='average':
                result = "{:.2f} {}/s".format(data[name][field] / duration, unit)
            elif mapping=='none':
                result = "{:.2f} {}".format(data[name][field], unit)
            results.append(result)
        result_string += '\t'.join(results)
        #print(result_string)

    results = []
    for m in metrics:
        field = m[0]
        unit = m[1]
        if reduction=='sum':
            reduced = 0
            for node in nodes:
                name = node[0]
                reduced += data[name][field]
            if mapping=='average':
                result = "{:.2f} {}/s".format(reduced / duration, unit)
            elif mapping=='none':
                result = "{:.2f} {}".format(reduced, unit)
            results.append(result)
            result_string = "Total: "
        elif reduction=='average':
            reduced = 0
            for node in nodes:
                name = node[0]
                reduced += data[name][field]
            if mapping=='average':
                result = "{:.2f} {}/s".format(reduced / duration / len(nodes), unit)
            elif mapping=='none':
                result = "{:.2f} {}".format(reduced / len(nodes), unit)
            results.append(result)
            result_string = "Average: "
    result_string += '\t'.join(results)
    print(result_string)

print_node_results("Transaction generation", [('generated_transactions', 'Txs'), ('generated_transaction_bytes', 'B')], 'sum')
print_node_results("Transaction confirmation (++/--)", [('confirmed_transactions', 'Txs'), ('confirmed_transaction_bytes', 'B'), ('deconfirmed_transactions', 'Txs'), ('deconfirmed_transaction_bytes', 'B')], 'average')
print_node_results("Transaction block confirmation (++/--)", [('confirmed_transaction_blocks', 'Blks'), ('deconfirmed_transaction_blocks', 'Blks')], 'average')
print_node_results("Block processed (proposer, voter, transaction)", [('processed_proposer_blocks', 'Blks'), ('processed_voter_blocks', 'Blks'), ('processed_transaction_blocks', 'Blks')], 'average')
print_node_results("Block mined (proposer, voter, transaction)", [('mined_proposer_blocks', 'Blks'), ('mined_voter_blocks', 'Blks'), ('mined_transaction_blocks', 'Blks')], 'sum')
print_node_results("Block propogation delay mean (proposer, voter, transaction)", [('proposer_block_delay_mean', 'ms'), ('voter_block_delay_mean', 'ms'), ('transaction_block_delay_mean', 'ms')], 'average', 'none')
print_node_results("Incoming message queue length", [('incoming_message_queue', 'Msgs')], 'average', 'none')

print("Transaction generation failures: {:.2f} Txs".format(tx_gen_fails))


