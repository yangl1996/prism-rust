#!/bin/bash
function waitportopen()
{
	while ! nc -z localhost $1; do
		sleep 0.2
	done
}

function etcdget()
{
	local dt=''
	until dt=`etcdctl get $1`
	do
		sleep 0.2
	done
	echo $dt
}

function killandassert()
{
	kill $1
	while kill -0 $1; do
		sleep 0.2
	done
}

# create config files
python3 bootstrap.py

# start btcd, lnd, and etcd
btcd &> /dev/null &
btcd_pid=$!
lnd --noseedbackup &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &

# wait for etcd to start
waitportopen 2379

# wait for btcd to start
waitportopen 18556

# wait for lnd to start
waitportopen 10009

# store ip in etcd
etcdctl set "/$NODENAME/ip" "$NODEIP"

# create btc wallet and store address in etcd
btc_addr=`lncli -n simnet newaddress np2wkh | jq -r '.address'`
etcdctl set "/$NODENAME/btcaddr" $btc_addr

# if we are the mining node, mine coins for each node
miner_node=`cat default_topo.json | jq -r '.miner'`
if [ "$NODENAME" == "$miner_node" ]
then
	for node in `cat default_topo.json | jq -r '.nodes | keys[]'`; do
		# wait for the node to publish its btc address
		node_btcaddr=`etcdget /$node/btcaddr`

		# kill current btcd instance and wait for it to exit
		killandassert $btcd_pid

		# start btcd and set mining addr
		btcd --miningaddr=$node_btcaddr &> /dev/null &
		btcd_pid=$!

		# wait for btcd to restart
		waitportopen 18556

		# mine blocks
		btcctl --simnet --rpcuser=btcd --rpcpass=btcd generate 400
	done
fi

# store public key in etcd
pubkey=`lncli -n simnet getinfo | jq -r '.identity_pubkey'`
etcdctl set "/$NODENAME/pubkey" $pubkey

# establish channel with peers
for chan in `cat default_topo.json | jq -c '.lnd_channels | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	if [ "$NODENAME" == "$src" ]
	then
		peer_pubkey=`etcdget /$dst/pubkey`
		peer_ip=`etcdget /$dst/ip`
		lncli -n simnet connect $peer_pubkey@$peer_ip:9735
	fi
done

# enter interactive bash
bash

