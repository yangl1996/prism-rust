#!/bin/bash
apt-get update
apt-get install apt-transport-https ca-certificates curl software-properties-common -y
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo apt-key add -
add-apt-repository "deb [arch=amd64] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable"
apt-get update
apt-get install docker-ce -y
usermod -aG docker ubuntu
snap install go --classic
apt-get install build-essential git -y
apt-get install -y software-properties-common
add-apt-repository -y ppa:bitcoin/bitcoin
apt-get install -y libssl-dev libevent-dev libboost-system-dev libboost-filesystem-dev libboost-chrono-dev libboost-test-dev libboost-thread-dev build-essential libtool autotools-dev automake pkg-config bsdmainutils python3 libzmq3-dev libdb4.8-dev libdb4.8++-dev
