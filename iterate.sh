#!/bin/bash
docker stop s0
docker stop s1
docker stop s2
docker stop s3
docker stop s4
docker rm s0
docker rm s1
docker rm s2
docker rm s3
docker rm s4
docker build -t test .
docker run -itd --name s0 -e NODENAME=0 -e NODEIP=10.0.1.100 --network spider --ip 10.0.1.100 test
docker run -itd --name s1 -e NODENAME=1 -e NODEIP=10.0.1.101 --network spider --ip 10.0.1.101 test
docker run -itd --name s2 -e NODENAME=2 -e NODEIP=10.0.1.102 --network spider --ip 10.0.1.102 test
docker run -itd --name s3 -e NODENAME=3 -e NODEIP=10.0.1.103 --network spider --ip 10.0.1.103 test
docker run -itd --name s4 -e NODENAME=4 -e NODEIP=10.0.1.104 --network spider --ip 10.0.1.104 test
