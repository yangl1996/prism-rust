import os
import default_topo as t
import templates

nodename = os.getenv("NODENAME")
nodeip = t.nodes[nodename]

btcd_peers = []

for conn in t.btcd_connection:
    if conn[0] == nodename:
        peer = conn[1]
        peer_ip = t.nodes[peer]
        btcd_peers.append(peer_ip)

btcd_config_string = templates.btcd_conf

for peer in btcd_peers:
    btcd_config_string += templates.btcd_connect.format(peer)

os.makedirs("/root/.btcd", exist_ok=True)
with open("/root/.btcd/btcd.conf", "w") as f:
    f.write(btcd_config_string)

os.makedirs("/root/.lnd", exist_ok=True)
with open("/root/.lnd/lnd.conf", "w") as f:
    f.write(templates.lnd_conf)

etcd_nodes = []
for k, v in t.nodes.items():
    etcd_nodes.append("{}=http://{}:2380".format(k ,v))
etcd_nodes_string = ','.join(etcd_nodes)
etcd_config_string = templates.etcd_conf.format(nodename, nodeip, nodeip, etcd_nodes_string)
os.makedirs("/root/.etcd", exist_ok=True)
with open("/root/.etcd/etcd.conf", "w") as f:
    f.write(etcd_config_string)
