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

function monitorpendingchannels()
{
	local has_pending=''
	while true
	do
		if has_pending=`etcdctl get /cluster/haspendingchan` ; then
			if [ "$has_pending" == "init" ] ; then
				# still init
				sleep 4.5
			elif [ "$has_pending" == "yes" ] ; then
				# has pending channels
				btcctl --simnet --rpcuser=btcd --rpcpass=btcd generate 6
				sleep 4.5
			fi
		else
			break
		fi
	done
}

function generatepayreq()
{
	local invoice=''
	local interval=`awk "BEGIN{print 1.0/$3}"`
	while true
	do
		invoice=`lncli -n simnet addinvoice --amt 100000 | jq -r '.pay_req'`
		etcdctl set "/payments/$1/$2" $invoice
		sleep $interval
	done
}

function watchpayreq()
{
	while true
	do
		etcdctl exec-watch "/payments/$1/$2" -- sh -c 'lncli -n simnet sendpayment -f --pay_req=$ETCD_WATCH_VALUE &'
	done
}

function killintime()
{
	sleep $2
	kill $1
}

# create config files
python3 bootstrap.py

# start btcd, lnd, and etcd
btcd &> /dev/null &
btcd_pid=$!
lnd --noseedbackup --debughtlc &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &

# wait for etcd to start
waitportopen 2379

# wait for btcd to start
waitportopen 18556

# wait for lnd to start
waitportopen 10009

# store ip in etcd
etcdctl set "/nodeinfo/$NODENAME/ip" "$NODEIP"
etcdctl set /cluster/haspendingchan init

# create btc wallet and store address in etcd
btc_addr=`lncli -n simnet newaddress np2wkh | jq -r '.address'`
etcdctl set "/nodeinfo/$NODENAME/btcaddr" $btc_addr

# if we are the mining node, mine coins for each node
miner_node=`cat default_topo.json | jq -r '.miner'`
if [ "$NODENAME" == "$miner_node" ]
then
	for node in `cat default_topo.json | jq -r '.nodes | .[] | .name'`; do
		# wait for the node to publish its btc address
		node_btcaddr=`etcdget /nodeinfo/$node/btcaddr`

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
etcdctl set "/nodeinfo/$NODENAME/pubkey" $pubkey

# establish channel with peers
for chan in `cat default_topo.json | jq -c '.lnd_channels | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	if [ "$NODENAME" == "$src" ]
	then
		# establish p2p connection
		peer_pubkey=`etcdget /nodeinfo/$dst/pubkey`
		peer_ip=`etcdget /nodeinfo/$dst/ip`
		lncli -n simnet connect $peer_pubkey@$peer_ip:9735

		# establish channel
		# we need to retry until succeed because btcd might by syncing
		funding_output=''
		until funding_output=`lncli -n simnet openchannel --node_key=$peer_pubkey --local_amt=2000000 --push_amt=1000000`
		do
			sleep 0.5
		done

		# publish on etcd
		funding_txid=`echo $funding_output | jq -r '.funding_txid'`
		etcdctl set "/channels/$src/$dst" "$funding_txid"
	fi
done

# miner node should mine blocks after all channels has been established
if [ "$NODENAME" == "$miner_node" ]
then
	monitorpendingchannels &
fi

# monitor how many pending channels are there
while true
do
	pending_chans=`lncli -n simnet pendingchannels | jq '.pending_open_channels | length'`
	if (( $pending_chans == 0 )) ; then
		break
	fi
	# if there are still channels pending, tell the miner
	# this info will live for 5 sec. The miner checks this key
	# every (<5) sec, so it will always be seen by the miner
	etcdctl set --ttl=5 /cluster/haspendingchan yes
	sleep 5
done

# wait for itself to receive all channels
num_channels=`cat default_topo.json | jq '.lnd_channels | length'`
until [ `lncli -n simnet getnetworkinfo | jq '.num_channels'` == "$num_channels" ]
do
	sleep 1
done
etcdctl set "/nodeinfo/$NODENAME/seenallchans" "yes"

# wait for all nodes to receive all channels
for node in `cat default_topo.json | jq -r '.nodes | .[] | .name'`; do
	etcdget /nodeinfo/$node/seenallchans
done

# enter interactive bash
bash

