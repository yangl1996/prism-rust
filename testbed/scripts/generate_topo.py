#!/usr/bin/python3
import sys
import json

num_nodes = int(sys.argv[1])
topology = sys.argv[2]

nodes = []
connections = []

if topology == "clique":
    for i in range(0, num_nodes):
        nodes.append("node_" + str(i))
        for j in range(0, i):
            connections.append({"from": nodes[i], "to": nodes[j]})
else:
    print("Unrecognized topology")
    sys.exit(1)

result = {"nodes": nodes, "connections": connections}
print(json.dumps(result, sort_keys=True, indent=4))
