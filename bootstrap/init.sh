#!/bin/bash
python3 bootstrap.py
btcd &> /dev/null &
lnd &> /dev/null &
etcd --config-file ~/.etcd/etcd.conf &> /dev/null &
bash
