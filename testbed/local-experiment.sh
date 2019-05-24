#!/bin/bash

if [ "$#" -ne 1 ]; then
	echo "Usage: ./local-experiment.sh <num of nodes>"
	exit 0
fi

trap kill_prism INT

function kill_prism() {
	for pid in $pids; do
		kill $pid
	done
}


binary_path=${PRISM_BINARY-../target/debug/prism}
num_nodes=$1

# generate keypairs and addresses
for (( i = 0 ; i < $num_nodes ; i++ )); do
	cmd="$binary_path keygen --addr"
	$cmd 2> ${i}.addr 1> ${i}.pkcs8
done

# build funding command
funding_cmd=""
for (( i = 0 ; i < $num_nodes ; i++ )); do
	addr=`cat ${i}.addr`
	funding_cmd="$funding_cmd --fund-addr $addr"
done

p2p_port=6000
api_port=7000
vis_port=8000

pids=""

for (( i = 0; i < $num_nodes; i++ )); do
	p2p=`expr $p2p_port + $i`
	api=`expr $api_port + $i`
	vis=`expr $vis_port + $i`
	command="$binary_path --p2p 127.0.0.1:${p2p} --api 127.0.0.1:${api} --visual 127.0.0.1:${vis} --blockdb /tmp/prism-${i}-blockdb.rocksdb --blockchaindb /tmp/prism-${i}-blockchaindb.rocksdb --utxodb /tmp/prism-${i}-utxodb.rocksdb --walletdb /tmp/prism-${i}-wallet.rocksdb --mine -vvv --load-key ${i}.pkcs8"

	for (( j = 0; j < $i; j++ )); do
		peer_port=`expr $p2p_port + $j`
		command="$command -c 127.0.0.1:${peer_port}"
	done

	command="$command $funding_cmd"
	$command &> ${i}.log &
	pid="$!"
	pids="$pids $pid"
	echo "Node $i started as process $pid"
done

for pid in $pids; do
	wait $pid
done

echo "Experiment terminated"
