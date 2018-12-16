#!/bin/bash
python3 bootstrap.py
btcd &> /dev/null &
btcd_pid=$!
lnd --noseedbackup &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &

# wait for etcd to start
while ! nc -z localhost 2379; do
	sleep 0.1
done
etcdctl set "/$NODENAME/ip" "$NODEIP"
bash
