#!/usr/bin/python3
import sys
if len(sys.argv) < 2:
    print('Please add the input topology json file')
    sys.exit(1)

import json
import networkx as nx
import matplotlib.pyplot as plt

topology = sys.argv[1]

G = nx.DiGraph()
nodes = []
public_ips = []
with open("nodes.txt") as fin:
    for line in fin:
        split = line.strip().split(',')
        # add the node in format "node_x"
        G.add_node(split[0])
        nodes.append(split[0])
        # add the public ip
        public_ips.append(split[2])

with open(topology) as fin:
    topo = json.load(fin)
assert set(topo['nodes'])==set(nodes), 'nodes file and topology file not compatible'
for edge in topo['connections']:
    G.add_edge(edge['from'], edge['to'])
    G.add_edge(edge['to'], edge['from'])

# you can pick one layout in these lines
# pos = nx.spring_layout(G)
pos = nx.spectral_layout(G)
# pos = nx.shell_layout(G)

# draw ip plot
plt.figure(figsize=(12,9))

nx.draw_networkx(G, pos, with_labels=False, edge_color='gray')
plt.axis('off')
    
for node, ip in zip(nodes, public_ips):
    x,y = pos[node]
    #plt.text(x,y+0.16,s=node, fontsize=14, horizontalalignment='center')
    plt.text(x,y+0.06,s=ip, fontsize=14, horizontalalignment='center')

# plot a legend showing ips
# legends = '\n'.join(['{:7}: {}'.format(n,p) for n,p in zip(nodes, public_ips)])
# plt.annotate(legends, xy=(1,0.1), xycoords='axes fraction', fontsize=14)

plt.tight_layout()

plt.savefig('topology.png')
plt.savefig('topology.svg')

