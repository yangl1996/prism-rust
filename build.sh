#!/bin/bash
export BUILDROOT="$PWD"
export BTCPATH="$PWD/btcroot"
export GOPATH="$PWD/goroot"
export PATH="$PATH:$GOPATH/bin"
export BRANCH='new-stats'

BUILD_LND='false'
BUILD_EXPCTRL='false'
BUILD_BITCOIND='false'
BUILD_ETCDJQ='false'
# check all command line arguments
for var in "$@"
do
	case $var in
		lnd)
			BUILD_LND='true' ;;
		expctrl)
			BUILD_EXPCTRL='true' ;;
		bitcoind)
			BUILD_BITCOIND='true' ;;
		etcdjq)
			BUILD_ETCDJQ='true' ;;
	esac
done

if [ "$BUILD_LND" == "true" ] || [ "$BUILD_EXPCTRL" == "true" ] ; then
	go get -u github.com/golang/dep/cmd/dep
	go get -d github.com/urfave/cli
	go get -d github.com/vibhaa/lnd

	rm -rf $GOPATH/src/github.com/lightningnetwork/lnd
	mv $GOPATH/src/github.com/vibhaa/lnd $GOPATH/src/github.com/lightningnetwork

	cd $GOPATH/src/github.com/lightningnetwork/lnd
	git checkout $BRANCH
	git pull origin $BRANCH
	git apply $BUILDROOT/patches/*.lndpatch
fi

if [ "$BUILD_LND" == "true" ] ; then
	echo "Building lnd"
	cd $GOPATH/src/github.com/lightningnetwork/lnd
	make && make install
fi

if [ "$BUILD_EXPCTRL" == "true" ] ; then
	echo "Building experiment controller"
	cp -r $BUILDROOT/expctrl $GOPATH/src/github.com/lightningnetwork/lnd
	cd $GOPATH/src/github.com/lightningnetwork/lnd/expctrl
	go get -d go.etcd.io/etcd/client
	go build
fi

if [ "$BUILD_BITCOIND" == "true" ] ; then
	echo "Building bitcoind"
	mkdir -p $BTCPATH
	cd $BTCPATH
	git clone 'https://github.com/bitcoin/bitcoin.git'
	cd "$BTCPATH/bitcoin"
	git checkout 'v0.17.1'
	git apply $BUILDROOT/patches/*.btcpatch
	$BTCPATH/bitcoin/autogen.sh
	$BTCPATH/bitcoin/configure
	make -j4
fi

if [ "$BUILD_ETCDJQ" == "true" ]; then
	echo "Download etcd and jq"
	mkdir -p $BUILDROOT/downloads
	wget 'https://github.com/etcd-io/etcd/releases/download/v3.3.10/etcd-v3.3.10-linux-amd64.tar.gz' -O $BUILDROOT/downloads/etcd.tar.gz
	tar -xf $BUILDROOT/downloads/etcd.tar.gz -C $BUILDROOT/downloads

	wget 'https://github.com/stedolan/jq/releases/download/jq-1.6/jq-linux64' -O $BUILDROOT/downloads/jq
	chmod +x $BUILDROOT/downloads/jq
fi

echo "Gathering binaries to $BUILDROOT/binaries/"
mkdir -p $BUILDROOT/binaries
if [ "$BUILD_ETCDJQ" == "true" ]; then
	cp $BUILDROOT/downloads/jq $BUILDROOT/binaries/
	cp $BUILDROOT/downloads/etcd-v3.3.10-linux-amd64/etcd* $BUILDROOT/binaries/
fi
if [ "$BUILD_LND" == "true" ]; then
	cp $GOPATH/bin/* $BUILDROOT/binaries/
fi
if [ "$BUILD_EXPCTRL" == "true" ]; then
	cp $GOPATH/src/github.com/lightningnetwork/lnd/expctrl/expctrl $BUILDROOT/binaries/
fi
if [ "$BUILD_BITCOIND" == "true" ]; then
	cp $BTCPATH/bitcoin/src/bitcoind $BUILDROOT/binaries/
	cp $BTCPATH/bitcoin/src/bitcoin-cli $BUILDROOT/binaries/
	cp $BTCPATH/bitcoin/src/bitcoin-tx $BUILDROOT/binaries/
fi

echo "Cleaning up build files"
rm -rf $GOPATH
rm -rf $BTCPATH
rm -rf $BUILDROOT/downloads

