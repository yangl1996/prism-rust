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
