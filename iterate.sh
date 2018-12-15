#!/bin/bash
docker stop spider1
docker stop spider2
docker stop spider3
docker rm spider1
docker rm spider2
docker rm spider3
docker build -t test .
docker run -itd --name spider1 -e NODENAME=spider1 --network spider --ip 10.0.0.100 test
docker run -itd --name spider2 -e NODENAME=spider2 --network spider --ip 10.0.0.101 test
docker run -itd --name spider3 -e NODENAME=spider3 --network spider --ip 10.0.0.102 test
