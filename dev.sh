#!/bin/bash
docker stop spider1
docker stop spider2
docker stop spider3
docker rm spider1
docker rm spider2
docker rm spider3
docker build -t test .
docker run -itd --name spider1 -v /home/spider/spider-docker/bootstrap:/root/bootstrap -e NODENAME=spider1 -e NODEIP=10.0.1.100 --network spider --ip 10.0.1.100 test
docker run -itd --name spider2 -v /home/spider/spider-docker/bootstrap:/root/bootstrap -e NODENAME=spider2 -e NODEIP=10.0.1.101 --network spider --ip 10.0.1.101 test
docker run -itd --name spider3 -v /home/spider/spider-docker/bootstrap:/root/bootstrap -e NODENAME=spider3 -e NODEIP=10.0.1.102 --network spider --ip 10.0.1.102 test
