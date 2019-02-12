#!/bin/bash

echo "Cloning Scorex source code"
git clone https://github.com/bagavi/Scorex.git /home/ubuntu/scorex
cd /home/ubuntu/scorex
git checkout f13093e876fe37626bcc

pids=""
echo "Launching Scorex instances"
for config in /home/ubuntu/payload/scorex-configs/*.conf; do
	echo "Launching $(basename $config .conf)"
	sbt "project examples" "runMain examples.bitcoin.BitcoinApp $config" &> "/home/ubuntu/log/$(basename $config .conf).log" &
	pids="$pids $!"
done

echo "All nodes launched, the script will wait for all Scorex processes to exit"
for pid in $pids; do
	wait $pid
done

