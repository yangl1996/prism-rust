#!/bin/bash

# create config files
python3 bootstrap.py

# start btcd, lnd, and etcd
btcd &> /dev/null &
btcd_pid=$!
lnd --noseedbackup &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &

# wait for etcd to start
while ! nc -z localhost 2379; do
	sleep 0.1
done

# store ip in etcd
etcdctl set "/$NODENAME/ip" "$NODEIP"

# create btc wallet and store address in etcd
btc_addr=`lncli -n simnet newaddress np2wkh | jq -r '.address'`
etcdctl set "/$NODENAME/btcaddr" $btc_addr

# enter interactive bash
bash
