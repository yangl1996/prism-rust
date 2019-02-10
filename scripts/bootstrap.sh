#!/bin/bash

mkdir -p /home/ubuntu/log

echo "Bootstraping etcd"
bash /home/ubuntu/payload/bootstrap-etcd.sh
