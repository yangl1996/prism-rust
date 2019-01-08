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
	tot=`etcdget /payments/$1/$2/total`
	succ=`etcdget /payments/$1/$2/success`
	rate=`awk "BEGIN {print $succ/$tot}"`
	total_tot=`awk "BEGIN {print $total_tot+$tot}"`
	total_succ=`awk "BEGIN {print $total_succ+$succ}"`
	echo "$1->$2: Total=$tot, Success=$succ, Rate=$rate"
}

for chan in `cat $TOPO_FILE | jq -c '.demands | .[]'`; do
	src=`echo $chan | jq -r '.src'`
	dst=`echo $chan | jq -r '.dst'`
	echo `getresult $src $dst`
	echo `awk "BEGIN {print $total_succ/$total_tot }"`
done
