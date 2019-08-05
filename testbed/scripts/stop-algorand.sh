#!/bin/bash

echo "Killing algod and kmd processes"

pkill algod
wait $!
pkill kmd
wait $!

echo "Resetting Algorand data files"
rm -rf /home/ubuntu/payload/algorand-nodedata
tar xf ~/local.tar.gz -C /home/ubuntu/payload
