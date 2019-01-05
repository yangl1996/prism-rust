#!/bin/bash
function etcdget()
{
	local dt=''
	until dt=`etcdctl get $1`
	do
		sleep 0.2
	done
	echo $dt
}

function getresult()
{
	local tot=''
	local succ=''
	tot=`etcdget /payments/$1/$2/total`
	succ=`etcdget /payments/$1/$2/success`
	rate=`awk "BEGIN {print $succ/$tot}"`
	echo "$1->$2: Total=$tot, Success=$succ, Rate=$rate"
}

for chan in `cat $TOPO_FILE | jq -c '.demands | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	echo `getresult $src $dst`
done
