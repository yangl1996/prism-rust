#!/bin/bash

if [ "$#" -lt "4" ]; then
	echo "Usage: ./start.sh <node name> <node port> <api port> <visual port> <peer port> ..."
	exit 0
fi

binary_path=${PRISM_BINARY-../target/debug/prism}
node_name=$1
node_port=$2
api_port=$3
visual_port=$4
shift 4	# pop the first arg

command="$binary_path --p2p 127.0.0.1:${node_port} --api 127.0.0.1:${api_port} --blockdb /tmp/${node_name}-blockdb.rocksdb --blockchaindb /tmp/${node_name}-blockchain.rocksdb --utxodb /tmp/${node_name}-utxodb.rocksdb --walletdb /tmp/${node_name}-wallet.rocksdb --mine -vvvv --visual 127.0.0.1:${visual_port}"

for port in "$@"; do
	command="$command -c 127.0.0.1:${port}"
done

eval $command

