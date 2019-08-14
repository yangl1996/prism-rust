#!/bin/bash
DEFAULT_LAUNCH_TEMPLATE=lt-07c210f12b766840c
DEFAULT_REGION='us-west-1'

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
	local instances=`aws ec2 run-instances --launch-template LaunchTemplateId=$DEFAULT_LAUNCH_TEMPLATE --count $1 --query 'Instances[*].InstanceId' | jq -r '. | join(" ")'`
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
		echo "$instance,$ip,$lan,$DEFAULT_REGION" >> instances.txt
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

declare -a regions=("eu-north-1" "ap-south-1" "eu-west-3" "eu-west-2" "eu-west-1" "ap-northeast-2" "ap-northeast-1" "me-south-1" "ca-central-1" "ap-east-1" "ap-southeast-1" "ap-southeast-2" "eu-central-1" "us-east-1" "us-east-2" "us-west-1" "us-west-2")

declare -a launch_template_ids=("lt-074165339867aa834" "lt-0e85ca27c485a13ce" "lt-0b77f96fe8631b010" "lt-0228fc0f23033acf5" "lt-00f97677e33e94706" "lt-0e380a9ccc0c4276a" "lt-0bd3ab2ea92f9c1f1" "lt-0d75559e93ea6b22d" "lt-094dfaed17f258c8d" "lt-0837a4136e9862849" "lt-0a154de3c17fb82ef" "lt-04153cb112577a813" "lt-009f97b35d4f1de3d" "lt-0d9bcde49095f337d" "lt-0e50969fb1afb62f2" "lt-07c210f12b766840c" "lt-0a7e1261dc0dca417")

function start_instances_global
{
	echo "Really?"
	select yn in "Yes" "No"; do
		case $yn in
			Yes ) break ;;
			No ) echo "Nothing happened."; exit ;;
		esac
	done
	rm -f instances.txt
	rm -f ~/.ssh/config.d/prism
	input="config.txt"
	while IFS= read -r line
	do
		reg="$(cut -d',' -f1 <<<"$line")"
		instances="$(cut -d',' -f2 <<<"$line")"
		region_flag=false
		index=0
		for region in "${regions[@]}"
		do
			if [ "$reg" == "$region" ]; then
				region_flag=true
				break
			fi
			index=$((index+1))
		done
		if [ "$region_flag" = false ]; then
			tput setaf 1
			echo "Region must be one of the following:"
			echo ${regions[*]}
			tput sgr0
			exit 1
		fi
		start_region_instances ${region} ${instances} ${launch_template_ids[$index]}
	done < "$input"
}

function start_region_instances
{
	echo "Launching AWS EC2 $2 instances at $1"
	local instances=`aws ec2 run-instances --launch-template LaunchTemplateId=$3 --region $1 --count $2 --query 'Instances[*].InstanceId' | jq -r '. | join(" ")'`
	echo "Querying public IPs and writing to SSH config"
	while [ 1 ]
	do
		rawdetails=`aws ec2 describe-instances --instance-ids $instances --region $1 --query 'Reservations[*].Instances[*].{publicip:PublicIpAddress,id:InstanceId,privateip:PrivateIpAddress}[]'`
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
		echo "$instance,$ip,$lan,$1" >> instances.txt
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
	curl -s --form-string "token=$PUSHOVER_TOKEN" --form-string "user=$PUSHOVER_USER" --form-string "title=EC2 Instances Launched" --form-string "message=$2 EC2 instances were just launched by user $(whoami)." https://api.pushover.net/1/messages.json &> /dev/null
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
		IFS=',' read -r id ip lan reg <<< "$instance"
		instance_ids="$instance_ids $id"
		aws ec2 terminate-instances --instance-ids $id --region $reg > log/aws_stop.log
	done
	echo "Terminating instances $instance_ids"
	tput setaf 2
	echo "Instances terminated"
	tput sgr0
	curl -s --form-string "token=$PUSHOVER_TOKEN" --form-string "user=$PUSHOVER_USER" --form-string "title=EC2 Instances Stopped" --form-string "message=EC2 instances launched at $(date -r instances.txt) were just terminated by user $(whoami)." https://api.pushover.net/1/messages.json &> /dev/null
	pkill python3.7
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
	#scp prism:~/prism/target/release/prism-copy payload/common/binary/prism
	cp ../target/release/prism payload/common/binary/prism
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

function sync_payload
{
	#echo "Uploading payload to S3"
	#aws s3 rm --quiet --recursive s3://prism-binary/payload
	#aws s3 sync --quiet payload s3://prism-binary/payload
	echo "Downloading payload on each instance"
	execute_on_all get_payload
}

function get_payload_single
{
	ssh $1 -- "rm -f /home/ubuntu/*.tar.gz && rm -rf /home/ubuntu/payload && mkdir -p /home/ubuntu/payload"
	echo "Deleted payload"
	rsync  payload/$1.tar.gz $1:/home/ubuntu/payload
	rsync  payload/common.tar.gz $1:/home/ubuntu/payload
	echo "Synced payload"
    ssh $1 -- "mv /home/ubuntu/payload/$1.tar.gz /home/ubuntu/payload/local.tar.gz && tar xf /home/ubuntu/payload/local.tar.gz -C /home/ubuntu/payload && tar xf /home/ubuntu/payload/common.tar.gz -C /home/ubuntu/payload"
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
	ssh $1 -- 'sudo rm -rf /tmp/prism && sudo mkdir -m 777 /tmp/prism && sudo mkfs -F -t ext4 /dev/nvme0n1 && sudo mount /dev/nvme0n1 /tmp/prism && sudo chmod 777 /tmp/prism'
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

function join_by
{
	local IFS="$1"
	shift
	echo "$*"
}

function add_traffic_shaping_single
{
	# the $2: latency, $3: throughput
	local ports=`cat nodes.txt | grep $1 | cut -f 5 -d ,`
	local port_list=`join_by ',' $ports`
	ssh $1 -- "sudo /home/ubuntu/payload/binary/comcast --device=ens5 --latency=$2 --target-bw=$3 --target-port=$port_list"
}

function remove_traffic_shaping_single
{
	ssh $1 -- "sudo /home/ubuntu/payload/binary/comcast --device=ens5 --stop"
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
	curl -s "http://$3:$4/miner/start?lambda=60000&lazy=false"
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
	sed -i "s/host:[^,]*/host: 'ec2-54-183-248-97.us-west-1.compute.amazonaws.com'/g" ../visualization/prism/relay_server.js
	sed -i "s/ws:\/\/[^:]*/ws:\/\/ec2-54-183-248-97.us-west-1.compute.amazonaws.com/g" ../visualization/prism/client.js
	node ../visualization/prism/relay_server.js > /dev/null 2>&1 &
	python3.7 -m http.server 5000 --directory ../visualization/prism > /dev/null 2>&1 &
	run_experiment
	echo "Demo Started"
	python3.7 -m http.server 3001 > /dev/null 2>&1 &
	pkill grafana-rrd-server
	#~/go/bin/grafana-rrd-server -r data/ -s 1 &
	./telematics/telematics log -duration 7200 -grafana
}

mkdir -p log
case "$1" in
	help)
		cat <<- EOF
		Helper script to run Prism distributed tests

		Manage AWS EC2 Instances

		  start-instances n     Start n EC2 instances
		  start-instances-global Start EC2 instances from ./config.txt
		  stop-instances        Terminate EC2 instances
		  install-tools         Install tools
		  fix-config            Fix SSH config
		  mount-ramdisk         Mount RAM disk
		  unmount-ramdisk       Unmount RAM disk
		  mount-nvme            Mount NVME 
		  unmount-nvme          Unmount NVME

		Run Experiment

		  gen-payload topo      Generate scripts and configuration files
		  build [nostrip]	Build the Prism client binary
		  sync-payload          Synchronize payload to remote servers
		  start-prism           Start Prism nodes on each remote server
		  stop-prism            Stop Prism nodes on each remote server
		  run-exp               Run the experiment
		  show-demo             Start the demo workflow
		  stop-tx               Stop generating transactions
		  stop-mine             Stop mining
		  shape-traffic l b     Limit the throughput to b Kbps and add latency of l ms
		  reset-traffic         Remove the traffic shaping filters

		Collect Data
		  
		  get-perf              Get performance data
		  show-vis              Open the visualization page for the given node
		  profile node f d      Capture stack trace for node with frequency f and duration d
		  flamegraph node       Generate and download flamegraph for node
		  open-dashboard        Open the performance dashboard

		Connect to Testbed

		  run-all cmd           Run command on all instances
		  ssh i                 SSH to the i-th server (1-based index)
		  scp i src dst         Copy file from remote
		  read-log node         Read the log of the given node
		EOF
		;;
	start-instances)
		start_instances $2 ;;
	start-instances-global)
		start_instances_global ;;
	stop-instances)
		stop_instances ;;
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
