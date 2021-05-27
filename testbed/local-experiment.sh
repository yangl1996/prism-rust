#!/bin/bash

VOTER_CHAINS="1000"
MINING_RATE="0.05"
THROUGHPUT="10.0"
MINING_MODIFIER="1"	# mine faster than it should be

if [ "$#" -ne 1 ]; then
	echo "Usage: ./local-experiment.sh <num of nodes>"
	exit 0
fi

sudo mkdir -p /prism
sudo umount -f -q /prism
sudo mount -t tmpfs -o size=2000m tmpfs /prism
sudo chmod 777 /prism

throughput_param=`echo "$THROUGHPUT / 1" | bc`
blkps=`echo "(${VOTER_CHAINS}.0 + 1.0) * ${MINING_RATE} + ${THROUGHPUT} / (64000.0 / 168.0)" | bc`
mining_lambda=`echo "1000000.0 / ${MINING_MODIFIER} / ( ${blkps} / ${1}.0 ) / 1" | bc`

echo "Throughput=${throughput_param}, Mining Lambda=${mining_lambda} (${blkps} blocks/s)"
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
		confirmed_blocks=`echo $result | jq .[$'"confirmed_transaction_blocks"']`
		total_latency=`echo $result | jq .[$'"total_transaction_block_confirmation_latency"']`
		echo "Node $i Latency: $(expr $total_latency / $confirmed_blocks) ms"
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
		wait $pid
	done
	sudo umount -f /prism
	sudo rm -rf /prism
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
	command="$binary_path --p2p 127.0.0.1:${p2p} --api 127.0.0.1:${api} --visual 127.0.0.1:${vis} --blockdb /prism/prism-${i}-blockdb.rocksdb --blockchaindb /prism/prism-${i}-blockchaindb.rocksdb --utxodb /prism/prism-${i}-utxodb.rocksdb --walletdb /prism/prism-${i}-wallet.rocksdb -vv --load-key ${i}.pkcs8 --fund-coins=100000 --voter-chains=${VOTER_CHAINS} --tx-throughput=${throughput_param} --proposer-mining-rate=${MINING_RATE} --voter-mining-rate=${MINING_RATE} --confirm-confidence=20.0 --adversary-ratio=0.20"

	for (( j = 0; j < $i; j++ )); do
		peer_port=`expr $p2p_port + $j`
		command="$command -c 127.0.0.1:${peer_port}"
	done

	command="$command $funding_cmd"
	export RUST_BACKTRACE=1
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
	url="localhost:${port}/transaction-generator/set-arrival-distribution?interval=30&distribution=uniform"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to set transaction rate for node $i"
		exit 1
	fi
	url="localhost:${port}/transaction-generator/start?throttle=10000"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to start transaction generation for node $i"
		exit 1
	fi
	url="localhost:${port}/miner/start?lambda=${mining_lambda}&lazy=false"
	curl "$url" &> /dev/null
	if [ "$?" -ne 0 ]; then
		echo "Failed to start mining for node $i"
		exit 1
	fi
done

start_time=`date +%s`
echo "Running experiment, ^C to stop"

for pid in $pids; do
	wait $pid
done

echo "Experiment terminated"
for (( i = 0; i < $num_nodes; i++ )); do
	rm -f $i.log
	rm -f $i.addr
	rm -f $i.pkcs8
done
