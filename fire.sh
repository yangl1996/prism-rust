#!/bin/bash
hosts=(spider1)

function next_index()
{
	# current index
	local len=${#hosts[@]}
	local next=`expr $1 + 1`
	if [ "$next" -ge "$len" ]
	then
		next=0
	fi
	echo $next
}

function start_container()
{
	# name, ip, host
	ssh $3 -- docker run -itd --name "spider$1" -e NODENAME=$1 -e NODEIP=$2 -e SPIDER_EXP_NAME='hotnets-01' -e SPIDER_QUEUE=0 -e SPIDER_LOG_FIREBASE=1 --network spider --ip $2 spider
}

function destroy_container()
{
	# name, host
	ssh $2 -- docker stop "spider$1"
	ssh $2 -- docker rm "spider$1"
}

function build_container()
{
	# host
	ssh $1 -- git clone https://github.com/yangl1996/spider-docker.git
	ssh $1 -- docker build -t spider spider-docker
}

function init_swarm()
{
	# host
	local cmd_to_use=''
	cmd_to_use=`ssh $1 -- 'docker swarm init | sed -n 5p'`
	echo $cmd_to_use
}

function create_network()
{
	# host
	ssh $1 -- docker network create -d overlay --subnet 10.0.1.0/16 --attachable spider
}

# start swarm: docker swarm init
# create network: docker network create -d overlay --subnet 10.0.1.0/16 --attachable spider

function init()
{
	local cmd_to_use=`init_swarm ${hosts[0]}`
	for h in "${hosts[@]}"
	do
		if [ "$h" == "${hosts[0]}" ]
		then
			continue
		else
			ssh $h -- $cmd_to_use
		fi
	done

	create_network ${hosts[0]}

	for h in "${hosts[@]}"
	do
		 build_container $h &
	done
}

function start()
{
	local host_idx=0
	for node in `cat bootstrap/default_topo.json | jq -rc '.nodes | .[]'`
	do
		name=`echo $node | jq -r '.name'`
		ip=`echo $node | jq -r '.ip'`
		start_container $name $ip ${hosts[$host_idx]} &
		host_idx=`next_index $host_idx`
	done
}

function stop()
{
	local host_idx=0
	for node in `cat bootstrap/default_topo.json | jq -rc '.nodes | .[]'`
	do
		name=`echo $node | jq -r '.name'`
		destroy_container $name ${hosts[$host_idx]} 
		host_idx=`next_index $host_idx`
	done
}

