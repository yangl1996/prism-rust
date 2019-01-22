#!/bin/bash
function etcdget()
{
	local dt=''
	until dt=`etcdctl get $1 2> /dev/null`
	do
		sleep 0.3
	done
	echo $dt
}


function getNodeByPk
{
	# $1: public key
	for node in `cat $TOPO_FILE | jq -r '.nodes | .[] | .name'`; do
		pk=`etcdget /nodeinfo/$node/pubkey`
		if [ "$pk" == "$1" ]; then
			echo $node
		fi
	done
}

