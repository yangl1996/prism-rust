#!/bin/bash

function wait_for_line() {
	tail -F -n1000 $1 | grep -q "$2"
}

rm -rf /home/ubuntu/log
mkdir -p /home/ubuntu/log
mkdir -p /tmp/prism
rm -rf /tmp/prism/node*
cp -r /home/ubuntu/payload/algorand-nodedata/node* /tmp/prism

export ALGOD_ASSEMBLYDEADLINE=$1
export ALGOD_SMALLLAMBDA=$2
export ALGOD_BIGLAMBDA=$3
export ALGOD_RECOVERY_FREQ=$4
export ALGOD_BLOCKSIZE=$5

echo "Launching Algorand nodes"
for script in /home/ubuntu/payload/algorand-startup/*.sh; do
	[ -f "$script" ] || continue
	node_name=`basename $script .sh`
	echo "Launching $node_name"
	nohup bash $script &> /home/ubuntu/log/$node_name.log &
done

