#!/bin/bash
if [ "$1" = "on" ]; then
	if [ "$3" = "" ]; then
		qs="20"
	else
		qs="$3"
	fi
	echo "turning on bandwidth limiter to $2 Kbit/s queue size $qs"
	# reload the ts anchor
	sudo pfctl -e
	sudo pfctl -f /etc/pf.conf
	sudo pfctl -a ts -F all
	# create the dummynet pipe
	sudo dnctl pipe 1 config bw ${2}Kbit/s queue $qs delay 100
	sudo dnctl pipe 2 config bw ${2}Kbit/s queue $qs delay 100
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
