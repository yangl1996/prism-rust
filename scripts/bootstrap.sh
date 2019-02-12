#!/bin/bash

mkdir -p /home/ubuntu/log

echo "Bootstraping etcd"
bash /home/ubuntu/payload/bootstrap-etcd.sh

echo "Bootstraping sbt"
bash /home/ubuntu/payload/bootstrap-sbt.sh

echo "Launching Scorex nodes"
bash /home/ubuntu/payload/bootstrap-scorex.sh

