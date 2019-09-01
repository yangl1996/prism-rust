import sys
import os
import json
import subprocess

template = """
/home/ubuntu/payload/binary/prism --p2p {ip}:{p2p_port} --api {ip}:{api_port} --visual {ip}:{vis_port} --blockdb /tmp/prism/{node_name}-blockdb.rocksdb --blockchaindb /tmp/prism/{node_name}-blockchaindb.rocksdb --utxodb /tmp/prism/{node_name}-utxodb.rocksdb --walletdb /tmp/prism/{node_name}-wallet.rocksdb -vv --load-key /home/ubuntu/payload/prism-payload/{node_name}.pkcs8 {peer_opt} {fund_opt} --fund-coins=100000 --mempool-size=50000
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

# generate wallet keypair for each node
fund_addrs = []
prism_bin = "../target/debug/prism"
for name, node in nodes.items():
    result = subprocess.run([prism_bin, "keygen", "--addr"], capture_output=True, text=True)
    keypair = result.stdout
    address = result.stderr
    node["address"] = address.strip()
    fund_addrs.append("--fund-addr {}".format(address.strip()))
    os.makedirs("payload/{}/prism-payload".format(node['host']), exist_ok=True)
    with open("payload/{}/prism-payload/{}.pkcs8".format(node['host'], name), "w") as f:
        f.write(keypair.strip())
fund_opt = " ".join(fund_addrs)


# generate startup script for each node
for name, node in nodes.items():
    peers = []
    for c in topo['connections']:
        if c['from'] == name:
            dst = c['to']
            peers.append('-c {}:{}'.format(nodes[dst]['ip'], nodes[dst]['p2p_port']))
    peer_opt = ' '.join(peers)
    startup_str = template.format(
            node_name=name, ip=node['ip'], api_port=node['api_port'],
            p2p_port=node['p2p_port'], peer_opt=peer_opt,
            vis_port=node['vis_port'], fund_opt=fund_opt).strip()
    os.makedirs("payload/{}/prism-payload".format(node['host']), exist_ok=True)
    with open("payload/{}/prism-payload/{}.sh".format(node['host'], name), "w") as f:
        f.write(startup_str)

# write out node-host association
with open("nodes.txt", 'w') as f:
    for name, node in nodes.items():
        f.write("{},{},{},{},{},{},{}\n".format(name, node['host'], node['pubfacing_ip'], node['ip'], node['p2p_port'], node['api_port'], node['vis_port']))

