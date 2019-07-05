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

# you can pick one layout in these lines
# pos = nx.spring_layout(G)
# pos = nx.spectral_layout(G)
pos = nx.shell_layout(G)

# draw noip plot
plt.figure(figsize=(12,9))

nx.draw_networkx(G, pos, with_labels=False, edge_color='gray')
plt.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)
plt.tick_params(axis='y', which='both', right=False, left=False, labelleft=False)
plt.axis('off')
    
for node in nodes:
    x,y = pos[node]
    plt.text(x,y+0.1,s=node, fontsize=14, horizontalalignment='center')

plt.savefig('topology_noip.png')
plt.savefig('topology_noip.svg')

# draw ip plot
plt.figure(figsize=(12,9))

nx.draw_networkx(G, pos, with_labels=False, edge_color='gray')
plt.tick_params(axis='x', which='both', bottom=False, top=False, labelbottom=False)
plt.tick_params(axis='y', which='both', right=False, left=False, labelleft=False)
plt.axis('off')
    
for node, ip in zip(nodes, public_ips):
    x,y = pos[node]
    plt.text(x,y+0.16,s=node, fontsize=14, horizontalalignment='center')
    plt.text(x,y+0.08,s=ip, fontsize=14, horizontalalignment='center')

plt.savefig('topology.png')
plt.savefig('topology.svg')

