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

# wait for btcd to start
while ! nc -z localhost 18556; do
	sleep 0.1
done

# wait for lnd to start
while ! nc -z localhost 10009; do
	sleep 0.1
done

# store ip in etcd
etcdctl set "/$NODENAME/ip" "$NODEIP"

# create btc wallet and store address in etcd
btc_addr=`lncli -n simnet newaddress np2wkh | jq -r '.address'`
etcdctl set "/$NODENAME/btcaddr" $btc_addr

# restart btcd with mining address set to the new wallet 
echo "miningaddr=$btc_addr" >> /root/.btcd/btcd.conf
kill $btcd_pid
btcd &> /dev/null &

# wait for btcd to restart
while ! nc -z localhost 18556; do
	sleep 0.1
done

# enter interactive bash
bash
