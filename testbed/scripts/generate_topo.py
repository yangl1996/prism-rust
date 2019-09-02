#!/usr/bin/python3
import sys
import json
import networkx as nx

num_nodes = int(sys.argv[1])
topology = sys.argv[2]

nodes = []
connections = []

if topology == "clique":
    graph = nx.complete_graph(num_nodes)
elif topology == "randreg":
    degree = int(sys.argv[3])
    while True:
        graph = nx.random_regular_graph(degree, num_nodes)
        if nx.is_connected(graph):
            break
else:
    print("Unrecognized topology")
    sys.exit(1)

sys.stderr.write(str(nx.algorithms.distance_measures.diameter(graph)))

for node in graph.nodes():
    name = "node_" + str(node)
    nodes.append(name)
for edge in graph.edges():
    src = "node_" + str(edge[0])
    dst = "node_" + str(edge[1])
    connections.append({
        "from": src,
        "to": dst,
    })
result = {"nodes": nodes, "connections": connections}
print(json.dumps(result, sort_keys=True, indent=4))
