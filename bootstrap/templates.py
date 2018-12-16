btcd_conf = """simnet=1
rpcuser=btcd
rpcpass=btcd
listen=0.0.0.0
"""

btcd_connect = "connect={}\n"

lnd_conf = """[Application Options]
debuglevel=info

[Bitcoin]
bitcoin.simnet=1
bitcoin.active=1
bitcoin.node=btcd

[btcd]
btcd.rpcuser=btcd
btcd.rpcpass=btcd
"""

etcd_conf = """
name: '{}'
data-dir: '/root/.etcd/data'
wal-dir: '/root/.etcd/wal'
listen-peer-urls: 'http://{}:2380'
initial-advertise-peer-urls: 'http://{}:2380'
listen-client-urls: 'http://localhost:2379'
advertise-client-urls: 'http://localhost:2379'
initial-cluster-token: 'etcd-spider'
initial-cluster: '{}'
initial-cluster-state: 'new'
"""

