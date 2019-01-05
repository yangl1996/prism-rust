#!/bin/bash
export BUILDROOT="$PWD"
export GOPATH="$PWD/goroot"
export PATH="$PATH:$GOPATH/bin"
export BRANCH='stats'

echo "Building lnd and btcd"
go get -u github.com/golang/dep/cmd/dep
go get -d github.com/urfave/cli
go get -d github.com/vibhaa/lnd

rm -rf $GOPATH/src/github.com/lightningnetwork/lnd
mv $GOPATH/src/github.com/vibhaa/lnd $GOPATH/src/github.com/lightningnetwork

cd $GOPATH/src/github.com/lightningnetwork/lnd
git checkout $BRANCH
git pull origin $BRANCH
git apply $BUILDROOT/patches/*.patch
make && make install
make btcd

echo "Building experiment controller"
cp -r $BUILDROOT/expctrl $GOPATH/src/github.com/lightningnetwork/lnd
cd $GOPATH/src/github.com/lightningnetwork/lnd/expctrl
go get -d go.etcd.io/etcd/client
go build

echo "Download etcd and jq"
mkdir -p $BUILDROOT/downloads
wget 'https://github.com/etcd-io/etcd/releases/download/v3.3.10/etcd-v3.3.10-linux-amd64.tar.gz' -O $BUILDROOT/downloads/etcd.tar.gz
tar -xf $BUILDROOT/downloads/etcd.tar.gz -C $BUILDROOT/downloads

wget 'https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64' -O $BUILDROOT/downloads/jq
chmod +x $BUILDROOT/downloads/jq

echo "Gathering binaries to $BUILDROOT/binaries/"
mkdir -p $BUILDROOT/binaries
cp $BUILDROOT/downloads/jq $BUILDROOT/binaries/
cp $BUILDROOT/downloads/etcd-v3.3.10-linux-amd64/etcd* $BUILDROOT/binaries/
cp $GOPATH/bin/* $BUILDROOT/binaries/
cp $GOPATH/src/github.com/lightningnetwork/lnd/expctrl/expctrl $BUILDROOT/binaries/

echo "Cleaning up build files"
rm -rf $GOPATH
rm -rf $BUILDROOT/downloads
