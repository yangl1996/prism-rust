FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y python3 netcat ca-certificates software-properties-common
RUN apt-add-repository ppa:bitcoin/bitcoin -y
RUN apt-get install -y libssl1.1 libevent-2.1-6 libboost-system1.65.1 libboost-filesystem1.65.1 libboost-chrono1.65.1 libboost-test1.65.1 libboost-thread1.65.1 libzmq3-dev libdb4.8 libdb4.8++ libevent-pthreads-2.1-6
COPY binaries/* /usr/local/bin/
COPY scripts /root/scripts
COPY topology /root/topology
WORKDIR /root
CMD /root/scripts/entrypoint.sh

