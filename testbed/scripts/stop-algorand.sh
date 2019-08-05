#!/bin/bash

echo "Killing algod and kmd processes"

pkill algod
wait $!
pkill kmd
wait $!
