import os
import json
import templates

topo_file = os.getenv("TOPO_FILE")
with open(topo_file) as f:
    t = json.load(f)

nodename = os.getenv("NODENAME")
nodeip = os.getenv("NODEIP")

btcd_peers = []

nodes = {}

for item in t["nodes"]:
    nodes[item['name']] = item['ip']

for conn in t["btcd_connections"]:
    if conn["src"] == nodename:
        peer = conn["dst"]
        peer_ip = nodes[peer]
        btcd_peers.append(peer_ip)

bitcoind_config_string = templates.bitcoind_conf

for peer in btcd_peers:
    bitcoind_config_string += templates.bitcoind_connect.format(peer)

os.makedirs("/root/.bitcoin", exist_ok=True)
with open("/root/.bitcoin/bitcoin.conf", "w") as f:
    f.write(bitcoind_config_string)

os.makedirs("/root/.lnd", exist_ok=True)
with open("/root/.lnd/lnd.conf", "w") as f:
    f.write(templates.lnd_conf)

etcd_nodes = []
for k, v in nodes.items():
    etcd_nodes.append("{}=http://{}:2380".format(k ,v))
etcd_nodes_string = ','.join(etcd_nodes)
etcd_config_string = templates.etcd_conf.format(nodename, nodeip, nodeip, etcd_nodes_string)
os.makedirs("/root/.etcd", exist_ok=True)
with open("/root/.etcd/etcd.conf", "w") as f:
    f.write(etcd_config_string)
