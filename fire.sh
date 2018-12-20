#!/bin/bash
function start_container()
{
	# name, ip, host
	ssh $3 -- docker run -itd --name "spider$1" -e NODENAME=$1 -e NODEIP=$2 --network spider --ip $2 spider
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
	cmd_to_use=`init_swarm spider1`
	ssh spider2 -- $cmd_to_use
	ssh spider3 -- $cmd_to_use
	create_network spider1

	build_container spider1 &
	build_container spider2 &
	build_container spider3 &
}

function start()
{
	start_container 0 10.0.1.100 spider1 &
	start_container 1 10.0.1.101 spider1 &
	start_container 2 10.0.1.102 spider2 &
	start_container 3 10.0.1.103 spider2 &
	start_container 4 10.0.1.104 spider3 &
}

function stop()
{
	destroy_container 0 spider1 &
	destroy_container 1 spider1 &
	destroy_container 2 spider2 &
	destroy_container 3 spider2 &
	destroy_container 4 spider3 &
}

