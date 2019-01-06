bitcoind_conf = """regtest=1
server=1
listen=1
maxconnections=1024
zmqpubrawblock=tcp://127.0.0.1:28332
zmqpubrawtx=tcp://127.0.0.1:28333
rpcuser=bitcoind
rpcpassword=bitcoind
"""

bitcoind_connect = "connect={}\n"

lnd_conf = """[Application Options]
debuglevel=info
listen=0.0.0.0:9735
trickledelay=5000

[Bitcoin]
bitcoin.regtest=1
bitcoin.active=1
bitcoin.node=bitcoind
bitcoin.defaultchanconfs=0
bitcoin.defaultremotedelay=0

[bitcoind]
bitcoind.rpcuser=bitcoind
bitcoind.rpcpass=bitcoind
bitcoind.zmqpubrawblock=tcp://127.0.0.1:28332
bitcoind.zmqpubrawtx=tcp://127.0.0.1:28333
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

