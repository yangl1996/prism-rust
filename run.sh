#!/bin/bash

LAUNCH_TEMPLATE=lt-02226ebae5fbef5f3

function start_instances
{
	# $1: number of instances to start
	if [ $# -ne 1 ]; then
		tput setaf 1
		echo "Required: number of instances to start"
		tput sgr0
		exit 1
	fi
	echo "Really?"
	select yn in "Yes" "No"; do
		case $yn in
			Yes ) break ;;
			No ) echo "Nothing happened."; exit ;;
		esac
	done
	echo "Launching $1 AWS EC2 instances"
	aws ec2 run-instances --launch-template LaunchTemplateId=$LAUNCH_TEMPLATE --count $1 > log/aws_start.log
	local instances=`jq -r '.Instances[].InstanceId ' log/aws_start.log`
	echo "Waiting for network interfaces to attach"
	sleep 3
	rm -f instances.txt
	rm -f ~/.ssh/config.d/prism
	echo "Querying public IPs and writing to SSH config"
	for instance in $instances ;
	do
		local ip=`aws ec2 describe-instances --instance-ids $instance | jq -r '.Reservations[0].Instances[0].PublicIpAddress'`
		local lan=`aws ec2 describe-instances --instance-ids $instance | jq -r '.Reservations[0].Instances[0].PrivateIpAddress'`
		echo "$instance,$ip,$lan" >> instances.txt
		echo "Host $instance" >> ~/.ssh/config.d/prism
		echo "    Hostname $ip" >> ~/.ssh/config.d/prism
		echo "    User ubuntu" >> ~/.ssh/config.d/prism
		echo "    IdentityFile ~/.ssh/prism.pem" >> ~/.ssh/config.d/prism
		echo "    StrictHostKeyChecking no" >> ~/.ssh/config.d/prism
		echo "    UserKnownHostsFile=/dev/null" >> ~/.ssh/config.d/prism
		echo "" >> ~/.ssh/config.d/prism
	done
	tput setaf 2
	echo "Instance started, SSH config written"
	tput sgr0
}

function stop_instances
{
	echo "Really?"
	select yn in "Yes" "No"; do
		case $yn in
			Yes ) break ;;
			No ) echo "Nothing happened."; exit ;;
		esac
	done
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		instance_ids="$instance_ids $id"
	done
	echo "Terminating instances $instance_ids"
	aws ec2 terminate-instances --instance-ids $instance_ids > log/aws_stop.log
	tput setaf 2
	echo "Instances terminated"
	tput sgr0
}

function prepare_payload
{
	# $1: topology file to use
	if [ $# -ne 1 ]; then
		tput setaf 1
		echo "Required: topology file"
		tput sgr0
		exit 1
	fi
	echo "Deleting existing files"
	rm -rf payload
	mkdir -p payload
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		echo "Generating config files for $id"
		python3 scripts/gen_etcd_config.py $id $lan instances.txt
		cp scripts/bootstrap.sh payload/$id/bootstrap.sh
		cp scripts/bootstrap-etcd.sh payload/$id/bootstrap-etcd.sh
		cp scripts/bootstrap-sbt.sh payload/$id/bootstrap-sbt.sh
		cp scripts/bootstrap-scorex.sh payload/$id/bootstrap-scorex.sh
		cp scripts/start-scorex.sh payload/$id/start-scorex.sh
		cp scripts/stop-scorex.sh payload/$id/stop-scorex.sh
		cp scripts/get-scorex-perf.sh payload/$id/get-scorex-perf.sh
	done
	python3 scripts/gen_scorex_config.py instances.txt $1
	tput setaf 2
	echo "Payload written"
	tput sgr0
}

function sync_payload_single
{
	rsync -r payload/$1/ $1:/home/ubuntu/payload
}

function install_deps_single
{
	ssh $1 -- 'mkdir -p /home/ubuntu/log'
	ssh $1 -- 'bash /home/ubuntu/payload/bootstrap.sh &>/home/ubuntu/log/deps.log'
}

function start_scorex_single
{
	ssh $1 -- 'bash /home/ubuntu/payload/start-scorex.sh &>/home/ubuntu/log/start.log'
}

function stop_scorex_single
{
	ssh $1 -- 'bash /home/ubuntu/payload/stop-scorex.sh &>/home/ubuntu/log/stop.log'
}

function execute_on_all
{
	# $1: execute function '$1_single'
	# ${@:2}: extra params of the function
	local instances=`cat instances.txt`
	local pids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		echo "Executing $1 on $id"
		$1_single $id ${@:2} &>log/${id}_${1}.log &
		pids="$pids $!"
	done
	echo "Waiting for all jobs to finish"
	for pid in $pids ;
	do
		wait $pid
	done
	tput setaf 2
	echo "Finished"
	tput sgr0
}

function get_performance_metrics_single
{
	local perf
	perf=`ssh $2 -- "bash /home/ubuntu/payload/get-scorex-perf.sh $1" 2>/dev/null`
	echo "$1,$perf"
}

function get_protocol_metrics_single
{
	local perf
	perf=`curl -s "http://$3:$4/stats/txcountchain" | python3 scripts/get_num_trans.py`
	echo "$1,$perf"
}

function collect_data
{
	# $1: which function to execute
	local nodes=`cat nodes.txt`
	local pids=''
	rm -f perf.txt
	for node in $nodes; do
		local name
		local host
		local pubip
		local apiport
		IFS=',' read -r name host pubip _ apiport _ <<< "$node"
		$1_single $name $host $pubip $apiport > "${name}_data.txt" &
		pids="$pids $!"
	done
	for pid in $pids; do
		wait $pid
	done
	for node in $nodes; do
		local name
		IFS=',' read -r name _ <<< "$node"
		cat "${name}_data.txt"
	done
	rm *_data.txt
}

function run_on_all
{
	# $@: command to run
	local instances=`cat instances.txt`
	local pids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		echo "Job launched for $id"
		ssh $id -- "$@" &
		pids="$pids $!"
	done
	echo "Waiting for all jobs to finish"
	for pid in $pids ;
	do
		wait $pid
	done
	tput setaf 2
	echo "Finished"
	tput sgr0
}

function ssh_to_server
{
	# $1: which server to ssh to (starting from 1)
	if [ $# -ne 1 ]; then
		tput setaf 1
		echo "Required: the index of server to SSH to"
		tput sgr0
		exit 1
	fi
	local instance=`sed -n "$1 p" < instances.txt`
	local id
	local ip
	local lan
	IFS=',' read -r id ip lan <<< "$instance"
	tput setaf 2
	echo "SSH to $id at $ip"
	tput sgr0
	ssh $id
}

function scp_from_server
{
	# $1: server, $2: src path, $3: dst path
	if [ $# -ne 3 ]; then
		tput setaf 1
		echo "Required: the index of server, src path and dst path"
		tput sgr0
		exit 1
	fi
	local instance=`sed -n "$1 p" < instances.txt`
	local id
	local ip
	local lan
	IFS=',' read -r id ip lan <<< "$instance"
	cmd_to_run="scp -r ${id}:${2} $3"
	tput setaf 2
	echo "Executing $cmd_to_run"
	tput sgr0
	scp -r ${id}:${2} $3
}

function query_api
{
	# $1: which node to query
	# $2: which api to query
	if [ $# -ne 2 ]; then
		tput setaf 1
		echo "Required: node name and API endpoint"
		tput sgr0
		echo "API endpoints: blocktime, blockdiff, blocktxncount, blocktxn, balance"
		exit 1
	fi
	case "$2" in
		blocktime)
			endpoint="/stats/timechain" ;;
		blockdiff)
			endpoint="/stats/diffchain" ;;
		blocktxncount)
			endpoint="/stats/txcountchain" ;;
		blocktxn)
			endpoint="/debug/txchain" ;;
		balance)
			endpoint="/wallet/balances" ;;
		*)
			tput setaf 1
			echo "Unrecognized API endpoint"
			tput sgr0
			exit 1 ;;
	esac
	node=`cat nodes.txt | grep "$1,"`
	if [ $? != 0 ]; then
		tput setaf 1
		echo "Unrecognized node name"
		tput sgr0
		exit 1
	fi
	IFS=',' read -r name host pubip _ apiport _ <<< "$node"
	curl -s "http://$pubip:${apiport}${endpoint}"
}

mkdir -p log
case "$1" in
	help)
		cat <<- EOF
		Helper script to run Prism distributed tests

		Manage AWS EC2 Instances

		  start-instances n     Start n EC2 instances
		  stop-instances        Terminate EC2 instances

		Run Experiment

		  gen-payload topo      Generate scripts and configuration files
		  sync-payload          Synchronize payload to remote servers
		  install-deps          Install dependencies on remote servers
		  start-scorex          Start Scorex nodes on each remote server
		  stop-scorex           Stop Scorex nodes on each remote server
		  get-perf              Collect performance metrics
		  get-proto             Collect protocol metrics

		Connect to Testbed

		  run-all cmd           Run command on all instances
		  ssh i                 SSH to the i-th server (1-based index)
		  show node api         Query the API of a node
		  scp i src dst         Copy file from remote
		EOF
		;;
	start-instances)
		start_instances $2 ;;
	stop-instances)
		stop_instances ;;
	gen-payload)
		prepare_payload $2 ;;
	sync-payload)
		execute_on_all sync_payload ;;
	install-deps)
		execute_on_all install_deps ;;
	start-scorex)
		execute_on_all start_scorex ;;
	stop-scorex)
		execute_on_all stop_scorex ;;
	get-perf)
		collect_data get_performance_metrics ;;
	get-proto)
		collect_data get_protocol_metrics ;;
	run-all)
		run_on_all "${@:2}" ;;
	ssh)
		ssh_to_server $2 ;;
	show)
		query_api $2 $3 ;;
	scp)
		scp_from_server $2 $3 $4 ;;
	*)
		tput setaf 1
		echo "Unrecognized subcommand '$1'"
		tput sgr0 ;;
esac
