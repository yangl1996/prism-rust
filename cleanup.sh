#!/bin/bash
docker rm $(docker ps -a -q)
# Delete all images
docker rmi $(docker images -q)
