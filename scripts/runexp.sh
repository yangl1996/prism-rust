#!/bin/bash
function waitportopen()
{
	while ! nc -z localhost $1; do
		sleep 0.3
	done
}

function etcdget()
{
	local dt=''
	until dt=`etcdctl get $1 2> /dev/null`
	do
		sleep 0.3
	done
	echo $dt
}

function killandassert()
{
	kill $1
	wait $!
}

function monitorpendingchannels()
{
	local has_pending=''
	while true
	do
		if has_pending=`etcdctl get /cluster/haspendingchan` ; then
			if [ "$has_pending" == "init" ] ; then
				# still init
				sleep 0.8
			elif [ "$has_pending" == "yes" ] ; then
				# has pending channels
				bitcoin-cli -regtest generate 6 >>/root/log/bitcoin-cli.log 2>&1 
				sleep 0.8
			fi
		else
			sleep 0.8
		fi
	done
}

function waitforline()
{
	# $1: file to watch, $2: pid to monitor, ${@:3}: pid to monitor
	tail -F -n+0 --pid $2 $1 2>/dev/null | grep -qe "${@:3}"
}

echo "Generating config files for bitcoind, etcd and lnd"
# create config files
python3 /root/scripts/bootstrap.py

echo "Starting bitcoind, etcd and lnd"
# start bitcoind, lnd, and etcd

# start bitcoind
while true; do
	bitcoind &> /root/log/bitcoind.log &
	bitcoind_pid=$!
	# wait for bitcoind to start
	waitforline /root/log/bitcoind.log $bitcoind_pid 'addcon thread start'
	if [ $? == 1 ]; then
		# at this time, bitcoind has exited (in error)
		echo "Bitcoind did not start correctly"
	else
		echo "Bitcoind started"
		break
	fi
done
# 'connect' in config file won't work. add peers manually
bitcoind_config=`cat /root/.bitcoin/bitcoin.conf | grep 'connect='`
for config_line in $bitcoind_config; do
	peer_ip=${config_line: 8}
	bitcoin-cli -regtest addnode "$peer_ip" add >>/root/log/bitcoin-cli.log 2>&1
done

# start lnd
while true; do
	lnd --noseedbackup --debughtlc &> /root/log/lnd.log &
	lnd_pid=$!
	# wait for lnd to start
	waitforline /root/log/lnd.log $lnd_pid 'Opened wallet'
	if [ $? == 1 ]; then
		# at this time, lnd has exited (in error)
		echo "Lnd did not start correctly"
	else
		echo "Lnd started"
		break
	fi
done

# start etcd
while true; do
	etcd --config-file ~/.etcd/etcd.conf &> /root/log/etcd.log &
	etcd_pid=$!
	# wait for etcd to start
	waitforline /root/log/etcd.log $etcd_pid 'etcdserver: starting server'
	if [ $? == 1 ]; then
		# at this time, etcd has exited (in error)
		echo "Etcd did not start correctly"
	else
		echo "Etcd started"
		break
	fi
done

# store ip in etcd
echo "Publishing node name and ip address"
etcdctl set "/nodeinfo/$NODENAME/ip" "$NODEIP" &> /dev/null
etcdctl set /cluster/haspendingchan init &> /dev/null

# create btc wallet and store address in etcd
echo "Creating btc wallet"
btc_addr=`lncli -n regtest newaddress np2wkh | jq -r '.address'`
etcdctl set "/nodeinfo/$NODENAME/btcaddr" $btc_addr &> /dev/null

# if we are the mining node, mine coins for each node
miner_node=`cat $TOPO_FILE | jq -r '.miner'`
if [ "$NODENAME" == "$miner_node" ]
then
	mined_amt=100
	bitcoin-cli -regtest generate 100 >>/root/log/bitcoin-cli.log 2>&1
	for node in `cat $TOPO_FILE | jq -r '.nodes | .[] | .name'`; do
		# wait for the node to publish its btc address
		node_btcaddr=`etcdget /nodeinfo/$node/btcaddr`

		# mine blocks
		bitcoin-cli -regtest generatetoaddress 50 "$node_btcaddr" >>/root/log/bitcoin-cli.log 2>&1
		echo "Mined coins for $node"
		mined_amt=`expr $mined_amt + 50`
	done
	# in case we did't mine enough blocks
	to_mine=`expr 400 - $mined_amt`
	if [ "$to_mine" -gt "0" ]; then
		echo "Mining coins until we have mined 400"
		bitcoin-cli -regtest generate $to_mine >>/root/log/bitcoin-cli.log 2>&1
	fi
fi

# store public key in etcd
echo "Publishing lnd pubkey"
pubkey=`lncli -n regtest getinfo | jq -r '.identity_pubkey'`
etcdctl set "/nodeinfo/$NODENAME/pubkey" "$pubkey" &> /dev/null

# establish channel with peers
for chan in `cat $TOPO_FILE | jq -c '.lnd_channels | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	cap=`echo $chan | jq -r '.capacity'`
	if [ "$NODENAME" == "$src" ]
	then
		echo "Establishing P2P connection to $dst"
		# establish p2p connection
		peer_pubkey=`etcdget /nodeinfo/$dst/pubkey`
		peer_ip=`etcdget /nodeinfo/$dst/ip`
		lncli -n regtest connect $peer_pubkey@$peer_ip:9735 >>/root/log/lncli.log 2>&1

		# establish channel
		# we need to retry until succeed because btcd might by syncing
		echo "Creating channel to $dst"
		funding_output=''
		funding_amt=`expr $cap + $cap + 9050`
		until funding_output=`lncli -n regtest openchannel --node_key=$peer_pubkey --local_amt=$funding_amt --push_amt=$cap >>/root/log/lncli.log 2>&1`
		do
			sleep 0.5
		done

		# publish on etcd
		echo "Publishing the new channel"
		funding_txid=`echo $funding_output | jq -r '.funding_txid'`
		etcdctl set "/channels/$src/$dst" "$funding_txid" &> /dev/null
	fi
done

etcdctl set "/nodeinfo/$NODENAME/createdallchans" 'yes' &> /dev/null

# miner node should mine blocks after all channels has been established
if [ "$NODENAME" == "$miner_node" ]
then
	echo "Starting miner process"
	# this is because lnd refuses to create channel unless both ends have synced to the latest btc block
	# if we start mining before some node have created all channels, we may end up in a loop where blocks
	# are mined before it gets synced to both ends
	echo "Waiting for all nodes to have created all channels"
	for node in `cat $TOPO_FILE | jq -r '.nodes | .[] | .name'`; do
		echo "Waiting for node $node"
		etcdget /nodeinfo/$node/createdallchans &> /dev/null
	done
	monitorpendingchannels &> /dev/null &
fi

# monitor how many pending channels are there
echo "Waiting for all channels to get acknowledged"
while true
do
	pending_chans=`lncli -n regtest pendingchannels | jq '.pending_open_channels | length'`
	echo "Pending channels: $pending_chans"
	if (( $pending_chans == 0 )) ; then
		break
	fi
	# if there are still channels pending, tell the miner
	# this info will live for 5 sec. The miner checks this key
	# every (<5) sec, so it will always be seen by the miner
	etcdctl set --ttl=1 /cluster/haspendingchan yes &> /dev/null
	sleep 0.8
done

# wait for itself to receive all channels
echo "Waiting to see all channels"
num_channels=`cat $TOPO_FILE | jq '.lnd_channels | length'`
until [ `lncli -n regtest getnetworkinfo | jq '.num_channels'` == "$num_channels" ]
do
	sleep 1
done
etcdctl set "/nodeinfo/$NODENAME/seenallchans" "yes" &> /dev/null

# wait for all nodes to receive all channels
echo "Waiting for all nodes to see all channels"
for node in `cat $TOPO_FILE | jq -r '.nodes | .[] | .name'`; do
	echo "Waiting for node $node"
	etcdget /nodeinfo/$node/seenallchans &> /dev/null
done

echo "Running experiments"
expctrl &
mainpid=$!

sleep $EXP_TIME
killandassert $mainpid
killandassert $lnd_pid
killandassert $bitcoind_pid

# enter interactive bash
bash /root/scripts/getresults.sh

