import sys
import os
import json

template = """
scorex {{
  dataDir = /tmp/scorex/{node_name}/blockchain
  logDir = /tmp/scorex/{node_name}/log

  restApi {{
    bindAddress = "{addr}:{api_port}"
    api-key-hash = ""
  }}

  network {{
    nodeName = "generatorNode_{node_name}"
    bindAddress = "{addr}:{p2p_port}"
    knownPeers = [{peer_string}]
    agentName = "2-Hop"
  }}

  miner {{
    offlineGeneration = true
    targetBlockDelay = 20s
    blockGenerationDelay = 0ms
    rParamX10 = 8
    initialDifficulty = 20
    posAttachmentSize = 1
    blockNetworkTransmissionDelay = 0s
    minerNumber = "{miner_num}"
    txGenerationRate = 1s
    txsPerBlock = 100
  }}

  wallet {{
    seed = "{node_name}"
    password = "cookies"
    walletDir = "/tmp/scorex/{node_name}/wallet"
  }}
}}
"""

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
        next_free_port.append(10001)

# load nodes
with open(topology_file) as f:
    topo = json.load(f)

instance_idx = 0
instances_tot = len(instances)

nodes = {}

# assign ports and hosts for each node
for node in topo['nodes']:
    # iterate over all instances
    instance_idx += 1
    if instance_idx == instances_tot:
        instance_idx = 0
    this = {}
    this['host'] = instances[instance_idx][0]
    this['ip'] = instances[instance_idx][2]
    this['pubfacing_ip'] = instances[instance_idx][1]
    this['api_port'] = next_free_port[instance_idx]
    next_free_port[instance_idx] += 1
    this['p2p_port'] = next_free_port[instance_idx]
    next_free_port[instance_idx] += 1
    nodes[node['name']] = this

# generate config files for each node
miner_num = 1
for name, node in nodes.items():
    peers = []
    for c in topo['connections']:
        if c['src'] == name:
            dst = c['dst']
            peers.append('"{}:{}"'.format(nodes[dst]['ip'], nodes[dst]['p2p_port']))
    node['peer_str'] = ', '.join(peers)
    config_str = template.format(
            node_name=name, addr=node['ip'], api_port=node['api_port'],
            p2p_port=node['p2p_port'], peer_string=node['peer_str'],
            miner_num=miner_num)
    miner_num += 1
    os.makedirs("payload/{}/scorex-configs".format(node['host']), exist_ok=True)
    with open("payload/{}/scorex-configs/{}.conf".format(node['host'], name), "w") as f:
        f.write(config_str)

# write out node-host association
with open("nodes.txt", 'w') as f:
    for name, node in nodes.items():
        f.write("{},{},{},{},{},{}\n".format(name, node['host'], node['pubfacing_ip'], node['ip'], node['api_port'], node['p2p_port']))

