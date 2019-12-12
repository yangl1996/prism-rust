#!/bin/bash
if [ "$1" = "on" ]; then
	# calculate bdp
	qs=`echo "$2 / 8.0 * $3 / 1000.0 / 1" | bc`
	if [ "$qs" -ge "1000" ]; then
		echo "BDP=$qs, too big for dummynet"
		exit 1
	fi
	echo "turning on bandwidth limiter to $2 Kbit/s delay $3 ms queue size $qs Kbytes"
	# reload the ts anchor
	sudo pfctl -e
	sudo pfctl -f /etc/pf.conf
	sudo pfctl -a ts -F all
	# create the dummynet pipe
	sudo dnctl pipe 1 config bw ${2}Kbit/s queue ${qs}Kbytes delay $3
	sudo dnctl pipe 2 config bw ${2}Kbit/s queue ${qs}Kbytes delay $3
	sudo pfctl -a ts -f - <<FILTERS
dummynet out proto tcp from any to any port 6000 pipe 1
dummynet out proto tcp from any port 6000 to any pipe 2
FILTERS
elif [ "$1" = "off" ]; then
	echo "turning off bandwidth limiter"
	sudo pfctl -a ts -F all
	sudo dnctl -q flush
	sudo pfctl -d
else
	echo "Usage: ./bw.sh {on|off}"
fi
