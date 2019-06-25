#!/bin/bash

function wait_for_line() {
	tail -F -n1000 $1 | grep -q "$2"
}

mkdir -p /home/ubuntu/log
sudo rm -rf /tmp/prism/*

echo "Launching Prism nodes"
for script in /home/ubuntu/payload/prism-payload/*.sh; do
	[ -f "$script" ] || continue
	node_name=`basename $script .sh`
	echo "Launching $node_name"
	nohup bash $script &> /home/ubuntu/log/$node_name.log &
	echo "$!" >> /home/ubuntu/log/prism.pid
done

echo "Waiting for API server to start"
for script in /home/ubuntu/payload/prism-payload/*.sh; do
	[ -f "$script" ] || continue
	node_name=`basename $script .sh`
	wait_for_line /home/ubuntu/log/$node_name.log 'API server listening'
	echo "Node $node_name started"
done

echo "All nodes started. PIDs written to /home/ubuntu/log/prism.pid"
