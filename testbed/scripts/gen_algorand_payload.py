import sys
import os
import json
import shutil
import subprocess

instances_file = sys.argv[1]
instances = []
next_free_port = []

topology_file = sys.argv[2]
topo = {}

# load instances
with open(instances_file) as f:
    for line in f:
        i = line.rstrip().split(",")
        instances.append(i)
        next_free_port.append(6000)

# load nodes
with open(topology_file) as f:
    topo = json.load(f)

instance_idx = 0
instances_tot = len(instances)

nodes = {}

# assign ports and hosts for each node
for node in topo['nodes']:
    this = {}
    this['host'] = instances[instance_idx][0]
    this['ip'] = instances[instance_idx][2]
    this['pubfacing_ip'] = instances[instance_idx][1]
    this['p2p_port'] = next_free_port[instance_idx]
    next_free_port[instance_idx] += 1
    this['api_port'] = next_free_port[instance_idx]
    next_free_port[instance_idx] += 1
    this['vis_port'] = next_free_port[instance_idx]
    next_free_port[instance_idx] += 1
    nodes[node] = this
    # use the next instance
    instance_idx += 1
    if instance_idx == instances_tot:
        instance_idx = 0

# write out node-host association
with open("nodes.txt", 'w') as f:
    for name, node in nodes.items():
        f.write("{},{},{},{},{},{},{}\n".format(name, node['host'], node['pubfacing_ip'], node['ip'], node['p2p_port'], node['api_port'], node['vis_port']))

# copy node data for each instance and create node config for each node
num_nodes = len(nodes)
for name, node in nodes.items():
    os.makedirs("payload/{}/algorand-nodedata".format(node['host']), exist_ok=True)
    shutil.move('payload/staging/{}'.format(name), 'payload/{}/algorand-nodedata/{}'.format(node['host'], name))
    config = {
            "GossipFanout": num_nodes,
            "EndpointAddress": "{}:{}".format(node['ip'], node['api_port']),
            "NetAddress": "{}:{}".format(node['ip'], node['p2p_port']),
            "NodeExporterListenAddress": "{}:{}".format(node['ip'], node['vis_port']),
            "DNSBootstrapID": "",
            "TxPoolSize": 320000
            }
    with open('payload/{}/algorand-nodedata/{}/config.json'.format(node['host'], name), 'w') as f:
        json.dump(config, f, sort_keys=True, indent=4)

# create startup script for each node
template = '/home/ubuntu/payload/binary/goal node start -d /tmp/prism/{node_name} -p "{peer_address_list}" -l "{api_address}" && /home/ubuntu/payload/binary/goal kmd start -d /tmp/prism/{node_name}'
for name, node in nodes.items():
    os.makedirs("payload/{}/algorand-startup".format(node['host']), exist_ok=True)
    peer_addresses=[]
    for c in topo['connections']:
        if c['from'] == name:
            dst = c['to']
            peer_addresses.append("{}:{}".format(nodes[dst]['pubfacing_ip'], nodes[dst]['p2p_port']))
    peer_list = ';'.join(peer_addresses)
    startup_str = template.format(node_name=name, peer_address_list=peer_list, api_address=node['ip']+':'+str(node['api_port']))
    with open('payload/{}/algorand-startup/{}.sh'.format(node['host'], name), 'w') as f:
        f.write(startup_str)

