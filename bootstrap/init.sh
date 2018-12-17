#!/bin/bash

# create config files
python3 bootstrap.py

# start btcd, lnd, and etcd
btcd &> /dev/null &
btcd_pid=$!
lnd --noseedbackup &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &

# wait for etcd to start
while ! nc -z localhost 2379; do
	sleep 0.2
done

# wait for btcd to start
while ! nc -z localhost 18556; do
	sleep 0.2
done

# wait for lnd to start
while ! nc -z localhost 10009; do
	sleep 0.2
done

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
		until node_btcaddr=`etcdctl get /$node/btcaddr`
		do
			sleep 0.2
		done
		# kill current btcd instance and wait for it to exit
		kill $btcd_pid
		while kill -0 $btcd_pid; do
			sleep 0.2
		done
		# start btcd and set mining addr
		btcd --miningaddr=$node_btcaddr &> /dev/null &
		btcd_pid=$!
		# wait for btcd to restart
		while ! nc -z localhost 18556; do
			sleep 0.2
		done
		btcctl --simnet --rpcuser=btcd --rpcpass=btcd generate 400
	done
fi

# enter interactive bash
bash
