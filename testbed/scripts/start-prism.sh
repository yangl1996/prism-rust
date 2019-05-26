#!/bin/bash

mkdir -p /home/ubuntu/log
rm -rf /tmp/prism*

echo "Launching Prism nodes"
for script in /home/ubuntu/payload/prism-payload/*.sh; do
	node_name=`basename $script .sh`
	echo "Launching $node_name"
	nohup bash $script &> /home/ubuntu/log/$node_name.log &
	echo "$!" >> /home/ubuntu/log/prism.pid
done

echo "All nodes started. PIDs written to /home/ubuntu/log/prism.pid"
