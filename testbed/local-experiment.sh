#!/bin/bash

if [ "$#" -ne 1 ]; then
	echo "Usage: ./local-experiment.sh <num of nodes>"
	exit 0
fi
function wait_for_line() {
	# $1: file to watch, $2: line to watch
	tail -F -n1000 $1 | grep -q "$2"
}

function wait_for_line_bsd() {
	# $1: file to watch, $2: line to watch
	while true; do
		cat $1 | grep -q "$2"
		if [ "$?" -eq 0 ]; then
			break
		fi
		sleep 0.2
	done
}

trap kill_prism INT

function kill_prism() {
	echo "Collecting experiment data"
	end_time=`date +%s`
	elapsed=`expr $end_time - $start_time`

	generated=0
	generated_bytes=0
	generate_failures=0

	echo "------ Results ------"
	for (( i = 0; i < $num_nodes; i++ )); do
		port=`expr $api_port + $i`
		url="localhost:${port}/telematics/snapshot"
		result=`curl $url 2> /dev/null`
		generated=`expr $generated + $(echo $result | jq .[$'"generated_transactions"'])`
		generated_bytes=`expr $generated_bytes + $(echo $result | jq .[$'"generated_transaction_bytes"'])`
		generate_failures=`expr $generate_failures + $(echo $result | jq .[$'"generate_transaction_failures"'])`
		confirmed=`echo $result | jq .[$'"confirmed_transactions"']`
		confirmed_bytes=`echo $result | jq .[$'"confirmed_transaction_bytes"']`
		mined_proposer=`echo $result | jq .[$'"mined_proposer_blocks"']`
		mined_voter=`echo $result | jq .[$'"mined_voter_blocks"']`
		mined_transaction=`echo $result | jq .[$'"mined_transaction_blocks"']`
        mined=`expr $mined_proposer + $mined_voter + $mined_transaction`
		echo "Node $i Mined blocks: $(expr $mined / $elapsed) blk/s"
		echo "Node $i Transaction Confirmation: $(expr $confirmed / $elapsed) Tx/s"
		echo "Node $i Transaction Confirmation: $(expr $confirmed_bytes / $elapsed) B/s"
	done
	echo "Transaction Generation: $(expr $generated / $elapsed) Tx/s"
	echo "Transaction Generation: $(expr $generated_bytes / $elapsed) B/s"
	echo "Generation Failures: $generate_failures"
	echo "---------------------"

	for pid in $pids; do
		echo "Killing $pid"
		kill $pid
	done
}


binary_path=${PRISM_BINARY-../target/release/prism}
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

echo "Starting ${num_nodes} Prism nodes"
for (( i = 0; i < $num_nodes; i++ )); do
	p2p=`expr $p2p_port + $i`
	api=`expr $api_port + $i`
	vis=`expr $vis_port + $i`
	command="$binary_path --p2p 127.0.0.1:${p2p} --api 127.0.0.1:${api} --visual 127.0.0.1:${vis} --blockdb /tmp/prism-${i}-blockdb.rocksdb --blockchaindb /tmp/prism-${i}-blockchaindb.rocksdb --utxodb /tmp/prism-${i}-utxodb.rocksdb --walletdb /tmp/prism-${i}-wallet.rocksdb -vv --load-key ${i}.pkcs8"

	for (( j = 0; j < $i; j++ )); do
		peer_port=`expr $p2p_port + $j`
		command="$command -c 127.0.0.1:${peer_port}"
	done

	command="$command $funding_cmd"
	$command &> ${i}.log &
	pid="$!"
	pids="$pids $pid"
	wait_for_line_bsd "$i.log" 'P2P server listening'
	echo "Node $i started as process $pid"
done

echo "Waiting for all nodes to start"
for (( i = 0; i < $num_nodes; i++ )); do
	wait_for_line_bsd "$i.log" 'API server listening'
	echo "Node $i started"
done

echo "Starting transaction generation and mining on each node"
for (( i = 0; i < $num_nodes; i++ )); do
	port=`expr $api_port + $i`
	url="localhost:${port}/transaction-generator/set-arrival-distribution?interval=1000&distribution=uniform"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to set transaction rate for node $i"
		exit 1
	fi
	url="localhost:${port}/transaction-generator/start?throttle=50"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to start transaction generation for node $i"
		exit 1
	fi
	url="localhost:${port}/miner/start?lambda=50000&lazy=false"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to start mining for node $i"
		exit 1
	fi
done

start_time=`date +%s`
echo "Running experiment, ^C to stop"

if [ $1 -ne 1 ]; then
    echo "You can run the following command to compare two nodes' blockchain:"
    echo "python3 compare_blockchain.py localhost:$vis_port/blockchain.json?limit=10000?fork=true localhost:`expr $vis_port + 1`/blockchain.json?limit=10000?fork=true"
fi

for pid in $pids; do
	wait $pid
done

echo "Experiment terminated"
