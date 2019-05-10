#!/bin/bash

if [ "$#" -lt "2" ]; then
	echo "Usage: ./start.sh <node name> <node port> <peer port> ..."
	exit 0
fi

node_name=$1
node_port=$2
shift 2	# pop the first arg
binary_path=$PRISM_BINARY

command="$PRISM_BINARY --port ${node_port} --blockdb /tmp/${node_name}-blockdb.rocksdb --blockchain /tmp/${node_name}-blockchain.rocksdb --utxodb /tmp/${node_name}-utxodb.rocksdb --wallet /tmp/${node_name}-wallet.rocksdb --mine -vv"

for port in "$@"; do
	command="$command -c 127.0.0.1:${port}"
done

eval $command
