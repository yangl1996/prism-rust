#!/bin/bash

echo "Killing algod and kmd processes"

pkill algod
wait $!
pkill kmd
wait $!

echo "Resetting Algorand data files"
rm -rf /tmp/prism/node*
