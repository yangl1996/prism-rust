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
	local instances=""
	local remaining=$1
	while [ "$remaining" -gt "0" ]
	do
		if [ "10" -gt "$remaining" ]; then
			local thisbatch="$remaining"
		else
			local thisbatch="10"
		fi
		tput rc
		tput el
		echo -n "Remaining: $remaining, launching: $thisbatch"
		instances="$instances $(aws ec2 run-instances --launch-template LaunchTemplateId=$LAUNCH_TEMPLATE --count $thisbatch --query 'Instances[*].InstanceId' | jq -r '. | join(" ")')"
		remaining=`expr $remaining - $thisbatch`
	done
	tput rc
	tput el
	echo "Instances launched"
	rm -f instances.txt
	rm -f ~/.ssh/config.d/prism
	echo "Querying public IPs and writing to SSH config"
	while [ 1 ]
	do
		rawdetails=`aws ec2 describe-instances --instance-ids $instances --query 'Reservations[*].Instances[*].{publicip:PublicIpAddress,id:InstanceId,privateip:PrivateIpAddress}[]'`
		if echo $rawdetails | jq '.[].publicip' | grep null &> /dev/null ; then
			echo "Waiting for public IP addresses to be assigned"
			sleep 3
			continue
		else
			details=`echo "$rawdetails" | jq -c '.[]'`
			break
		fi
	done
	for instancedetail in $details;
	do
		local instance=`echo $instancedetail | jq -r '.id'`
		local ip=`echo $instancedetail | jq -r '.publicip'`
		local lan=`echo $instancedetail | jq -r '.privateip'`
		echo "$instance,$ip,$lan" >> instances.txt
		echo "Host $instance" >> ~/.ssh/config.d/prism
		echo "    Hostname $ip" >> ~/.ssh/config.d/prism
		echo "    User ubuntu" >> ~/.ssh/config.d/prism
		echo "    IdentityFile ~/.ssh/prism.pem" >> ~/.ssh/config.d/prism
		echo "    StrictHostKeyChecking no" >> ~/.ssh/config.d/prism
		echo "    UserKnownHostsFile=/dev/null" >> ~/.ssh/config.d/prism
		echo "" >> ~/.ssh/config.d/prism
	done
	echo "SSH config written, waiting for instances to initialize"
	aws ec2 wait instance-running --instance-ids $instances
	tput setaf 2
	echo "Instances started"
	tput sgr0
	curl -s --form-string "token=$PUSHOVER_TOKEN" --form-string "user=$PUSHOVER_USER" --form-string "title=EC2 Instances Launched" --form-string "message=$1 EC2 instances were just launched by user $(whoami)." https://api.pushover.net/1/messages.json &> /dev/null
}

function fix_ssh_config
{
	local instances=`jq -r '.Instances[].InstanceId ' log/aws_start.log`
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
	echo "SSH config written"
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
	curl -s --form-string "token=$PUSHOVER_TOKEN" --form-string "user=$PUSHOVER_USER" --form-string "title=EC2 Instances Stopped" --form-string "message=EC2 instances launched at $(date -r instances.txt) were just terminated by user $(whoami)." https://api.pushover.net/1/messages.json &> /dev/null
}

function count_instances
{
	result=`aws ec2 describe-instances --query 'Reservations[*].Instances[*].[InstanceId][][]' --filters Name=instance-state-name,Values=running Name=tag-key,Values=prism --output text`
	echo "$(echo $result | wc -w | tr -d ' ')"
}

function build_prism
{
	echo "Copying local repository to build machine"
	rsync -ar ../Cargo.toml prism:~/prism/
	rsync -ar ../src prism:~/prism/
	rsync -ar ../.cargo prism:~/prism/
	echo "Building Prism binary"
	ssh prism -- 'cd ~/prism && ~/.cargo/bin/cargo build --release' &> log/prism_build.log
	if [ $# -ne 1 ]; then
		echo "Stripping symbol"
		ssh prism -- 'cp ~/prism/target/release/prism ~/prism/target/release/prism-copy && strip ~/prism/target/release/prism-copy'
	else
		if [ "$1" = "nostrip" ]; then
			ssh prism -- 'cp ~/prism/target/release/prism ~/prism/target/release/prism-copy'
		else
			echo "Stripping symbol"
			ssh prism -- 'cp ~/prism/target/release/prism ~/prism/target/release/prism-copy && strip ~/prism/target/release/prism-copy'
		fi
	fi
	tput setaf 2
	echo "Finished"
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
	mkdir -p payload/common/binary
	mkdir -p payload/common/scripts

	echo "Download binaries"
	scp prism:~/prism/target/release/prism-copy payload/common/binary/prism
	cp scripts/start-prism.sh payload/common/scripts/start-prism.sh
	cp scripts/stop-prism.sh payload/common/scripts/stop-prism.sh

	echo "Generate etcd config files for each EC2 instance"
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		mkdir -p payload/$id
		python3 scripts/gen_etcd_config.py $id $lan instances.txt
	done

	echo "Generate prism config files and keypairs for each node"
	python3 scripts/gen_prism_payload.py instances.txt $1

	echo "Compressing payload files"
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		IFS=',' read -r id _ <<< "$instance"
		tar cvzf payload/$id.tar.gz -C payload/$id . &> /dev/null
		rm -rf payload/$id
	done
	tar cvzf payload/common.tar.gz -C payload/common . &> /dev/null
	rm -rf payload/common

	tput setaf 2
	echo "Payload written"
	tput sgr0
}

function prepare_algorand_payload
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
	mkdir -p payload/common/binary
	mkdir -p payload/common/scripts

	echo "Download binaries"
	cd algorand
	GOOS=linux GOARCH=amd64 go build
	cd ..
	scp algorand:~/go/bin/\{algod,algoh,algokey,carpenter,goal,kmd\} payload/common/binary/
	cp algorand/algorand payload/common/binary
	cp scripts/start-algorand.sh payload/common/scripts/start-algorand.sh
	cp scripts/stop-algorand.sh payload/common/scripts/stop-algorand.sh
	cp scripts/start-algorand-transaction.sh payload/common/scripts/start-algorand-transaction.sh
	cp scripts/stop-algorand-transaction.sh payload/common/scripts/stop-algorand-transaction.sh


	echo "Generate etcd config files for each EC2 instance"
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		mkdir -p payload/$id
		python3 scripts/gen_etcd_config.py $id $lan instances.txt
	done

	echo "Generating Algorand network template file"
	size=`cat $1 | jq '.nodes | length'`
	python3 scripts/gen_algorand_template.py $size > algorand_template.json

	echo "Generating and copying Algorand network data"
	scp algorand_template.json algorand:~
	ssh algorand -- 'rm -rf ~/eval && ~/go/bin/goal network create -r ~/eval -n eval -t ~/algorand_template.json' &> /dev/null
	rm -f algorand_template.json
	scp -r algorand:~/eval payload/staging &> /dev/null

	echo "Generating payload for each AWS EC2 instance"
	python3 scripts/gen_algorand_payload.py instances.txt $1

	echo "Compressing payload files"
	local instances=`cat instances.txt`
	local instance_ids=""
	for instance in $instances ;
	do
		local id
		IFS=',' read -r id _ <<< "$instance"
		tar cvzf payload/$id.tar.gz -C payload/$id . &> /dev/null
		rm -rf payload/$id
	done
	tar cvzf payload/common.tar.gz -C payload/common . &> /dev/null
	rm -rf payload/common
	rm -rf payload/staging
	rm -rf payload/scripts

	tput setaf 2
	echo "Payload written"
	tput sgr0
}

function sync_payload
{
	echo "Uploading payload to S3"
	aws s3 rm --quiet --recursive s3://prism-binary/payload
	aws s3 sync --quiet payload s3://prism-binary/payload
	echo "Downloading payload on each instance"
	execute_on_all get_payload
}

function get_payload_single
{
	ssh $1 -- "rm -f /home/ubuntu/*.tar.gz && rm -rf /home/ubuntu/payload && wget https://prism-binary.s3.amazonaws.com/payload/$1.tar.gz -O local.tar.gz && wget https://prism-binary.s3.amazonaws.com/payload/common.tar.gz && mkdir -p /home/ubuntu/payload && tar xf local.tar.gz -C /home/ubuntu/payload && tar xf common.tar.gz -C /home/ubuntu/payload"
}

function install_perf_single
{
	ssh $1 -- 'rm -f rustfilt && rm -rf inferno && sudo apt-get update -y && sudo apt-get install linux-tools-aws linux-tools-4.15.0-1032-aws binutils -y && wget https://github.com/yangl1996/rustfilt/releases/download/1/rustfilt && wget https://github.com/yangl1996/inferno/releases/download/bin/linux64.tar.gz && mkdir -p inferno && tar xf linux64.tar.gz -C inferno && chmod +x rustfilt && chmod +x inferno/* && sudo apt-get install -y c++filt && echo export PATH=$PATH:/home/ubuntu:/home/ubuntu/inferno >> /home/ubuntu/.profile'
}

function mount_tmpfs_single
{
	ssh $1 -- 'sudo rm -rf /tmp/prism && sudo mkdir -m 777 /tmp/prism && sudo mount -t tmpfs -o rw,size=20g tmpfs /tmp/prism'
}

function unmount_tmpfs_single
{
	ssh $1 -- 'sudo umount /tmp/prism && sudo rm -rf /tmp/prism'
}

function mount_nvme_single
{
	ssh $1 -- 'diskname=$(lsblk | grep 372 | cut -f 1 -d " ") && sudo rm -rf /tmp/prism && sudo mkdir -m 777 /tmp/prism && sudo mkfs -F -t ext4 /dev/$diskname && sudo mount /dev/$diskname /tmp/prism && sudo chmod 777 /tmp/prism'
}

function unmount_nvme_single
{
	ssh $1 -- 'sudo umount /tmp/prism && sudo rm -rf /tmp/prism'
}

function start_prism_single
{
	ssh $1 -- 'sudo mkdir -m 777 -p /tmp/prism && mkdir -p /home/ubuntu/log && bash /home/ubuntu/payload/scripts/start-prism.sh &>/home/ubuntu/log/start.log'
}

function stop_prism_single
{
	ssh $1 -- 'bash /home/ubuntu/payload/scripts/stop-prism.sh &>/home/ubuntu/log/stop.log'
}

function start_algorand_single
{
	ssh $1 -- "mkdir -p /home/ubuntu/log && bash /home/ubuntu/payload/scripts/start-algorand.sh $2 $3 $4 $5 $6 &>/home/ubuntu/log/start.log"
}

function stop_algorand_single
{
	ssh $1 -- 'bash /home/ubuntu/payload/scripts/stop-algorand.sh &>/home/ubuntu/log/stop.log'
}

function start_algorand_transaction_single
{
	ssh $1 -- "bash /home/ubuntu/payload/scripts/start-algorand-transaction.sh $2 &>/home/ubuntu/log/start-tx.log"
}

function stop_algorand_transaction_single
{
	ssh $1 -- 'bash /home/ubuntu/payload/scripts/stop-algorand-transaction.sh &>/home/ubuntu/log/stop-tx.log'
}

function join_by
{
	local IFS="$1"
	shift
	echo "$*"
}

function add_traffic_shaping_single
{
	# the $2: latency in ms, $3: throughput in kbps
	local common_ports='22 53 80 443'
	local ports=`cat nodes.txt | grep $1 | cut -f 6-7 -d , | tr , ' '`
	# calculate the bdp to determine the queue size
	qlen=`expr $3 \* $2 / 1500 / 8`
	# give some headroom to the queue size
	qlen=`expr $qlen \* 2`

	# deal with egress
	# add the root qdisc to the egress network interface and default traffic to class 10
	command="sudo tc qdisc add dev ens5 handle 10: root htb default 10 direct_qlen $qlen"
	# add the class for traffic with immunity (will be filtered to this class below) 
	command="$command && sudo tc class add dev ens5 parent 10: classid 10:1 htb rate 1000000kbit"
	# add the class for the rest of the traffic (assigned to this class by default)
	command="$command && sudo tc class add dev ens5 parent 10: classid 10:10 htb rate ${3}kbit"
	# add netem qdisc under class 10:10 to emulate delay
	command="$command && sudo tc qdisc add dev ens5 parent 10:10 handle 100: netem delay ${2}ms rate ${3}kbit limit $qlen"
	# filter out traffic that we don't want to be impacted and put it under 10:1
	for port in $ports; do
		# packets from all API/visualization servers
		command="$command && sudo tc filter add dev ens5 parent 10: protocol ip prio 1 u32 match ip sport $port 0xffff flowid 10:1"
	done
	for port in $common_ports; do
		# normal, innocent traffic: incoming/outgoing SSH, DNS, HTTP(S)
		command="$command && sudo tc filter add dev ens5 parent 10: protocol ip prio 1 u32 match ip sport $port 0xffff flowid 10:1"
		command="$command && sudo tc filter add dev ens5 parent 10: protocol ip prio 1 u32 match ip dport $port 0xffff flowid 10:1"
	done

	# deal with ingress
	# create an ifb device to which later we will install qdisc
	command="$command && sudo modprobe ifb"
	command="$command && sudo ifconfig ifb0 up"
	# add the qdisc to the ingress interface and forward all traffic to ifb
	command="$command && sudo tc qdisc add dev ens5 handle ffff: ingress"
	command="$command && sudo tc filter add dev ens5 parent ffff: protocol all u32 match u32 0 0 action mirred egress redirect dev ifb0"
	# install qdisc on the ifb device - now we can do w/ever we want on egress of ifb and it will apply to ingress
	# add the root device, and default all traffic to class 10 (the class we will punish)
	command="$command && sudo tc qdisc add dev ifb0 handle 10: root htb default 10 direct_qlen $qlen"
	# add the class for traffic with immunity
	command="$command && sudo tc class add dev ifb0 parent 10: classid 10:1 htb rate 1000000kbit"
	# add the class that we will punish, all traffic has been sent by default to this class
	command="$command && sudo tc class add dev ifb0 parent 10: classid 10:10 htb rate ${3}kbit"
	# add netem qdisc under 10:10 to install rate limiter
	command="$command && sudo tc qdisc add dev ifb0 parent 10:10 handle 100: netem rate ${3}kbit limit $qlen"
	# filter out traffic that we don't want to be impacted
	for port in $ports; do
		# packets going to all API/visualization servers
		command="$command && sudo tc filter add dev ifb0 parent 10: protocol ip prio 1 u32 match ip dport $port 0xffff flowid 10:1"
	done
	for port in $common_ports; do
		# normal, innocent traffic: incoming/outgoing SSH, DNS, HTTP(S)
		command="$command && sudo tc filter add dev ifb0 parent 10: protocol ip prio 1 u32 match ip sport $port 0xffff flowid 10:1"
		command="$command && sudo tc filter add dev ifb0 parent 10: protocol ip prio 1 u32 match ip dport $port 0xffff flowid 10:1"
	done
	
	ssh $1 -- "$command"
}

function remove_traffic_shaping_single
{
	ssh $1 -- "sudo tc qdisc del dev ens5 root && sudo tc qdisc del dev ens5 ingress && sudo tc qdisc del dev ifb0 root"
}

function tune_tcp_single
{
	ssh $1 -- "sudo sysctl -w net.core.rmem_max=50331648 && sudo sysctl -w net.core.wmem_max=50331648 && sudo sysctl -w net.ipv4.tcp_wmem='10240 87380 50331648' && sudo sysctl -w net.ipv4.tcp_rmem='10240 87380 50331648'"
}

function execute_on_all
{
	# $1: execute function '$1_single'
	# ${@:2}: extra params of the function
	local instances=`cat instances.txt`
	local pids=""
	echo "Executing $1"
	tput sc
	for instance in $instances ;
	do
		local id
		local ip
		local lan
		IFS=',' read -r id ip lan <<< "$instance"
		tput rc
		tput el
		echo -n "Executing $1 on $id"
		$1_single $id ${@:2} &>log/${id}_${1}.log &
		pids="$pids $!"
	done
	for pid in $pids ;
	do
		tput rc
		tput el
		echo -n "Waiting for job $pid to finish"
		if ! wait $pid; then
			tput rc
			tput el
			tput setaf 1
			echo "Task $pid failed"
			tput sgr0
			tput sc
		fi
	done
	tput rc
	tput el
	tput setaf 2
	echo "Finished"
	tput sgr0
}

function get_performance_single
{
	curl -s http://$3:$4/telematics/snapshot
}

function start_transactions_single
{
	curl -s "http://$3:$4/transaction-generator/set-arrival-distribution?interval=100&distribution=uniform"
	curl -s "http://$3:$4/transaction-generator/set-value-distribution?min=100&max=100&distribution=uniform"
	curl -s "http://$3:$4/transaction-generator/start?throttle=8000"
}

function start_mining_single
{
	curl -s "http://$3:$4/miner/start?lambda=300000&lazy=false"
}

function stop_transactions_single
{
	curl -s "http://$3:$4/transaction-generator/stop"
}

function stop_mining_single
{
	curl -s "http://$3:$4/miner/step"
}

function query_api 
{
	# $1: which data to get, $2: delay between nodes
	mkdir -p data
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		local pubip
		local apiport
		IFS=',' read -r name host pubip _ _ apiport _ <<< "$node"
		$1_single $name $host $pubip $apiport > "data/${name}_$1.txt" &
		pids="$pids $!"
		if [ "$2" -ne "0" ]; then
			sleep $2
		fi
	done
	for pid in $pids; do
		wait $pid
	done
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

function read_log
{
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		local pubip
		local apiport
		IFS=',' read -r name host pubip _ _ apiport _ <<< "$node"
		if [ $name == $1 ]; then
			ssh $host -- cat "/home/ubuntu/log/$name.log" | less
		fi
	done
}

function capture_stack_trace 
{
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		IFS=',' read -r name host _ <<< "$node"
		if [ $name == $1 ]; then
			command_string='sudo perf record -p $(pgrep -f /binary/prism.*'
			command_string="$command_string$1.*) -a --call-graph dwarf -F $2 -o /home/ubuntu/perf.data -- sleep $3"
			echo "$command_string" | ssh $host &> /dev/null
		fi
	done
}

function generate_flamegraph
{
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		IFS=',' read -r name host _ <<< "$node"
		if [ $name == $1 ]; then
			echo "sudo perf script -i perf.data | inferno-collapse-perf | rustfilt | c++filt | inferno-flamegraph > flame.svg" | ssh $host &> /dev/null
			scp "$host:~/flame.svg" .
		fi
	done
}

function open_dashboard
{
	# start grafana simple json data server
	~/go/bin/grafana-rrd-server -r data/ -s 1
	open 'http://localhost:3000/dashboard/script/prism.js?orgId=1&nodes=20'
}

function show_visualization
{
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		local pubip
		local visport 
		IFS=',' read -r name host pubip _ _ _ visport <<< "$node"
		if [ $name == $1 ]; then
			open "http://$pubip:$visport/"
		fi
	done
}

function show_performance
{
	local nodes=`cat nodes.txt`
	local pids=''
	for node in $nodes; do
		local name
		local host
		local pubip
		local visport 
		IFS=',' read -r name host pubip _ _ apiport _ <<< "$node"
		if [ $name == $1 ]; then
			curl "http://$pubip:$apiport/telematics/snapshot"
		fi
	done
}

function start_prism
{
	execute_on_all start_prism
	start_time=`date +%s`
}

function stop_prism
{
	execute_on_all stop_prism
	stop_time=`date +%s`
	echo "STOP $stop_time" >> experiment.txt
}

function run_experiment
{
	rm data/*
	echo "Starting Prism nodes"
	start_prism
	echo "All nodes started, starting transaction generation"
	query_api start_transactions 0
	query_api start_mining 0
	rm -f experiment.txt
	echo "START $start_time" >> experiment.txt
	echo "Running experiment"
}

function show_demo
{
	run_experiment
	echo "Demo Started"
	pkill grafana-rrd-server
	~/go/bin/grafana-rrd-server -r data/ -s 1 &
	./telematics/telematics log -duration 7200 -grafana
}

mkdir -p log
case "$1" in
	help)
		cat <<- EOF
		Helper script to run Prism distributed tests

		Manage AWS EC2 Instances

		  start-instances n          Start n EC2 instances
		  stop-instances             Terminate EC2 instances
		  count-instances            Count the running instances
		  install-tools              Install tools
		  fix-config                 Fix SSH config
		  mount-ramdisk              Mount RAM disk
		  unmount-ramdisk            Unmount RAM disk
		  mount-nvme                 Mount NVME 
		  unmount-nvme               Unmount NVME
		  shape-traffic l b          Limit the throughput to b Kbps and add latency of l ms
		  reset-traffic              Remove the traffic shaping filters
		  tune-tcp                   Set TCP parameters

		Run Experiment

		  gen-payload topo           Generate scripts and configuration files
		  build [nostrip]	     Build the Prism client binary
		  sync-payload               Synchronize payload to remote servers
		  start-prism                Start Prism nodes on each remote server
		  stop-prism                 Stop Prism nodes on each remote server
		  run-exp                    Run the experiment
		  show-demo                  Start the demo workflow
		  stop-tx                    Stop generating transactions
		  stop-mine                  Stop mining

		Run Algorand Experiment

		  gen-algorand topo          Generate config and data folders for Algorand
		  start-algorand d s b f s   Start Algorand nodes with deadline, small/big lambda, recovery freq, block size
		  stop-algorand              Stop Algorand nodes on each remote server
		  start-algorand-tx r        Start Algorand transactions on each remote server at rate r txn/s
		  stop-algorand-tx           Stop Algorand transactions on each remote server

		Collect Data
		  
		  get-perf                   Get performance data
		  show-vis                   Open the visualization page for the given node
		  profile node f d           Capture stack trace for node with frequency f and duration d
		  flamegraph node            Generate and download flamegraph for node
		  open-dashboard             Open the performance dashboard

		Connect to Testbed

		  run-all cmd                Run command on all instances
		  ssh i                      SSH to the i-th server (1-based index)
		  scp i src dst              Copy file from remote
		  read-log node              Read the log of the given node
		EOF
		;;
	start-instances)
		start_instances $2 ;;
	stop-instances)
		stop_instances ;;
	count-instances)
		count_instances ;;
	fix-config)
		fix_ssh_config ;;
	mount-ramdisk)
		execute_on_all mount_tmpfs ;;
	unmount-ramdisk)
		execute_on_all unmount_tmpfs ;;
	mount-nvme)
		execute_on_all mount_nvme ;;
	unmount-nvme)
		execute_on_all unmount_nvme ;;
	install-tools)
		execute_on_all install_perf ;;
	gen-payload)
		prepare_payload $2 ;;
	build)
		build_prism $2 ;;
	sync-payload)
		sync_payload ;;
	start-prism)
		start_prism ;;
	stop-prism)
		stop_prism ;;
	run-exp)
		run_experiment ;;
	show-demo)
		show_demo ;;
	stop-tx)
		query_api stop_transactions 0 ;;
	stop-mine)
		query_api stop_mining 0 ;;
	shape-traffic)
		execute_on_all add_traffic_shaping $2 $3 ;;
	reset-traffic)
		execute_on_all remove_traffic_shaping ;;
	tune-tcp)
		execute_on_all tune_tcp ;;
	gen-algorand)
		prepare_algorand_payload $2 ;;
	start-algorand)
		execute_on_all start_algorand $2 $3 $4 $5 $6;;
	stop-algorand)
		execute_on_all stop_algorand ;;
	start-algorand-tx)
		execute_on_all start_algorand_transaction $2 ;;
	stop-algorand-tx)
		execute_on_all stop_algorand_transaction ;;
	get-perf)
		show_performance $2 ;;
	show-vis)
		show_visualization $2 ;;
	profile)
		capture_stack_trace $2 $3 $4 ;;
	flamegraph)
		generate_flamegraph $2 ;;
	open-dashboard)
		open_dashboard ;;
	run-all)
		run_on_all "${@:2}" ;;
	ssh)
		ssh_to_server $2 ;;
	scp)
		scp_from_server $2 $3 $4 ;;
	read-log)
		read_log $2 ;;
	*)
		tput setaf 1
		echo "Unrecognized subcommand '$1'"
		tput sgr0 ;;
esac
