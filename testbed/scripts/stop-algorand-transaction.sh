#!/bin/bash
pkill -f 'algorand gentx'
wait $!
