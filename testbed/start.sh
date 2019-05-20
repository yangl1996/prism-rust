#!/bin/bash

if [ "$#" -lt "3" ]; then
	echo "Usage: ./start.sh <node name> <node port> <visual port> <peer port> ..."
	exit 0
fi

binary_path=${PRISM_BINARY-../target/debug/prism}
node_name=$1
node_port=$2
visual_port=$3
shift 3	# pop the first arg

command="$binary_path --port ${node_port} --blockdb /tmp/${node_name}-blockdb.rocksdb --blockchaindb /tmp/${node_name}-blockchain.rocksdb --utxodb /tmp/${node_name}-utxodb.rocksdb --walletdb /tmp/${node_name}-wallet.rocksdb --mine -vvvv --visual ${visual_port}"

for port in "$@"; do
	command="$command -c 127.0.0.1:${port}"
done

eval $command

#Example bash start.sh miner1 10001 127.0.0.1:8001
#        bash start.sh miner2 10002 127.0.0.1:8002 10001
