#!/bin/bash
function get_scorex_pid
{
	# $1: node name
	processes=`ps aux | grep java | grep -v runMain | grep $1`
	pids=''

	while read -r process
	do
		read -r user pid _ <<< "$process"
		pids="$pids $pid"
	done <<< "$processes"
	echo $pids
}

pids=`get_scorex_pid $1`

tot_cpu='0.0'
tot_mem='0.0'

for pid in $pids; do
	read -r cpu mem <<< `ps --no-headers -p $pid -o %cpu,%mem`
	tot_cpu=`awk "BEGIN {print $tot_cpu+$cpu}"`
	tot_mem=`awk "BEGIN {print $tot_mem+$mem}"`
done

echo "$tot_cpu,$tot_mem"
