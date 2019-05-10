#!/bin/bash

sudo rm -rf /tmp/scorex
mkdir -p /home/ubuntu/log

echo "Launching Scorex instances"
cd /home/ubuntu/scorex
for config in /home/ubuntu/payload/scorex-configs/*.conf; do
	echo "Launching $(basename $config .conf)"
	nohup sbt "project examples" "runMain examples.bitcoin.BitcoinApp $config" &> "/home/ubuntu/log/$(basename $config .conf).log" &
	echo "$!" >> /home/ubuntu/log/scorex.pid
done

echo "All nodes launched. PIDs written to ~/log/scorex.pid"

