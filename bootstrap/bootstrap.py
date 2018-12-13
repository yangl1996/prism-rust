import os
import default_topo as t
import templates

nodename = os.getenv("NODENAME")

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
