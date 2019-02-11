#!/bin/bash

echo "Cloning Scorex source code"
git clone https://github.com/bagavi/Scorex.git /home/ubuntu/scorex
cd /home/ubuntu/scorex

# sbt "project examples" "runMain examples.bitcoin.BitcoinApp src/main/resources/settings.conf"
echo ""
