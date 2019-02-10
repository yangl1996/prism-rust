#!/bin/bash

LAUNCH_TEMPLATE=lt-02226ebae5fbef5f3

function start_instances
{
	# $1: number of instances to start
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
		echo "$instance,$ip" >> instances.txt
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
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		IFS=',' read -r id ip <<< "$instance"
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
	echo "Deleting existing files"
	rm -rf payload
	mkdir -p payload
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		IFS=',' read -r id ip <<< "$instance"
		echo "Generating config files for $id"
		python3 scripts/gen_etcd_config.py $id $ip instances.txt
		cp scripts/bootstrap.sh payload/$id/bootstrap.sh
		cp scripts/bootstrap-etcd.sh payload/$id/bootstrap-etcd.sh
	done
	tput setaf 2
	echo "Payload written"
	tput sgr0
}

function sync_payload_single
{
	rsync -r payload/$1/ $1:/home/ubuntu/payload
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
		IFS=',' read -r id ip <<< "$instance"
		echo "Executing $1 on $id"
		$1_single $id ${@:2} &>log/${1}_${id}.log &
		pids="$pids $!"
	done
	echo "Waiting for all jobs to finish"
	for pid in $pids ;
	do
		wait $pid
	done
	tput setaf 2
	echo "Payload pushed to remote servers"
	tput sgr0
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
		IFS=',' read -r id ip <<< "$instance"
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
	local instance=`sed -n "$1 p" < instances.txt`
	local id
	local ip
	IFS=',' read -r id ip <<< "$instance"
	tput setaf 2
	echo "SSH to $id at $ip"
	tput sgr0
	ssh $id
}

mkdir -p log
case "$1" in
	help)
		cat <<- EOF
		Helper script to run Prism distributed tests

		Manage AWS EC2 Instances

			start-instances n
				Start n EC2 instances

			stop-instances
				Terminate EC2 instances

		Run Experiment

			gen-payload
				Generate scripts and configuration files

			sync-payload
				Synchronize payload to remote servers

		Connect to Testbed

			run-all cmd
				Run command on all instances

			ssh i
				SSH to the i-th server (1-based index)
		EOF
		;;
	start-instances)
		start_instances $2 ;;
	stop-instances)
		stop_instances ;;
	gen-payload)
		prepare_payload ;;
	sync-payload)
		execute_on_all sync_payload ;;
	run-all)
		run_on_all "${@:2}" ;;
	ssh)
		ssh_to_server $2 ;;
esac
