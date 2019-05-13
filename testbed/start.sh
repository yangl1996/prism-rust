#!/bin/bash

if [ "$#" -lt "2" ]; then
	echo "Usage: ./start.sh <node name> <node port> <peer port> ..."
	exit 0
fi

binary_path=${PRISM_BINARY-../target/debug/prism}
node_name=$1
node_port=$2
shift 2	# pop the first arg

command="$binary_path --port ${node_port} --blockdb /tmp/${node_name}-blockdb.rocksdb --blockchaindb /tmp/${node_name}-blockchain.rocksdb --utxodb /tmp/${node_name}-utxodb.rocksdb --walletdb /tmp/${node_name}-wallet.rocksdb --mine -vvv"

for port in "$@"; do
	command="$command -c 127.0.0.1:${port}"
done

eval $command
