#!/bin/bash
total_tot=0
total_succ=0

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
}

for chan in `cat $TOPO_FILE | jq -c '.demands | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	tot=`etcdget /payments/$src/$dst/total`
	succ=`etcdget /payments/$src/$dst/success`
	rate=`awk "BEGIN {print $succ/$tot}"`
	total_tot=`awk "BEGIN {print $total_tot+$tot}"`
	total_succ=`awk "BEGIN {print $total_succ+$succ}"`
	echo "$src->$dst: Total=$tot, Success=$succ, Rate=$rate"
done
echo `awk "BEGIN {print $total_succ/$total_tot}"`
