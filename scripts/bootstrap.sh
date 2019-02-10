#!/bin/bash

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


