import sys
import os

template = """
name: '{}'
data-dir: '/home/ubuntu/.etcd/data'
wal-dir: '/home/ubuntu/.etcd/wal'
listen-peer-urls: 'http://{}:2380'
initial-advertise-peer-urls: 'http://{}:2380'
listen-client-urls: 'http://localhost:2379'
advertise-client-urls: 'http://localhost:2379'
initial-cluster-token: 'etcd-prism'
initial-cluster: '{}'
initial-cluster-state: 'new'
"""

node_name = sys.argv[1]
node_ip = sys.argv[2]
instances_file = sys.argv[3]
instances = []
etcd_nodes = []
etcd_nodes_string = ""

with open(instances_file) as f:
    for line in f:
        i = line.rstrip().split(",")
        etcd_nodes.append("{}=http://{}:2380".format(i[0], i[2]))
    etcd_nodes_string = ','.join(etcd_nodes)

etcd_config_file = template.format(
        node_name, node_ip, node_ip, etcd_nodes_string)

os.makedirs("payload/{}".format(node_name), exist_ok=True)

with open("payload/{}/etcd.conf".format(node_name), "w") as f:
    f.write(etcd_config_file)
