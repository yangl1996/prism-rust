#!/bin/bash

function cleanup_docker_images
{
	echo "Removing all containers"
	docker rm $(docker ps -a -q)
	# Delete all images
	echo "Removing all images"
	docker rmi $(docker images -q)
}

function build_binaries
{
	cd $EXPROOT

	bash build.sh
}

function download_binaries
{
	cd $EXPROOT

	wget 'https://github.com/yangl1996/spider-docker/releases/download/v0.1/binaries.tar.gz'
	tar xf binaries.tar.gz
	rm binaries.tar.gz
}

function build_image
{
	cd $EXPROOT

	docker build -t spider .
}

USERHOME=/home/ubuntu
EXPROOT="${USERHOME}/spider-docker"
export PATH="$PATH:/snap/bin"

case "$1" in
	build_bin)
		build_binaries
		build_image ;;
	download_bin)
		download_binaries
		build_image ;;
	build_image)
		build_image ;;
	cleanup_image)
		cheanup_docker_images ;;
esac
