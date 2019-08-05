#!/bin/bash

function wait_for_line() {
	tail -F -n1000 $1 | grep -q "$2"
}

rm -rf /home/ubuntu/log
mkdir -p /home/ubuntu/log

echo "Launching Algorand nodes"
for script in /home/ubuntu/payload/algorand-startup/*.sh; do
	[ -f "$script" ] || continue
	node_name=`basename $script .sh`
	echo "Launching $node_name"
	nohup bash $script &> /home/ubuntu/log/$node_name.log &
done

