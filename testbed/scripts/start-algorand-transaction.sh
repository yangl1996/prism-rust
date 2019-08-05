#!/bin/bash

for script in /home/ubuntu/payload/algorand-startup/*.sh; do
	[ -f "$script" ] || continue
	node_name=`basename $script .sh`
	nohup /home/ubuntu/payload/binary/algorand gentx -rate $1 -node $node_name > /home/ubuntu/log/$node_name-tx.log & 
done

