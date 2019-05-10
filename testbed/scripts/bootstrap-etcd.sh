#!/bin/bash

function waitforline()
{
	# $1: file to watch, $2: pid to monitor, ${@:3}: text to look for
	tail -F -n+0 --pid $2 $1 2>/dev/null | grep -qe "${@:3}"
}

echo "Downloading etcd and jq"
mkdir -p /home/ubuntu/download
wget 'https://github.com/yangl1996/prism-testbed/releases/download/etcd-jq/binaries.tar.gz' -O /home/ubuntu/download/etcd-jq.tar.gz &>/dev/null
tar xf /home/ubuntu/download/etcd-jq.tar.gz -C /home/ubuntu/download &>/dev/null
sudo mv /home/ubuntu/download/binaries/etcd /usr/local/bin
sudo mv /home/ubuntu/download/binaries/etcdctl /usr/local/bin
sudo mv /home/ubuntu/download/binaries/jq /usr/local/bin
rm -rf /home/ubuntu/download/binaries

echo "Copying etcd configuration file"
mkdir -p /home/ubuntu/.etcd
cp /home/ubuntu/payload/etcd.conf /home/ubuntu/.etcd/etcd.conf

echo "Launching etcd"
while true; do
	etcd --config-file /home/ubuntu/.etcd/etcd.conf &> /home/ubuntu/log/etcd.log &
	etcd_pid=$!
	# wait for etcd to start
	waitforline /home/ubuntu/log/etcd.log $etcd_pid 'etcdserver: starting server'
	if [ $? == 1 ]; then
		# at this time, etcd has exited (in error)
		echo "Etcd did not start correctly, retrying"
	else
		echo "Etcd started"
		break
	fi
done

