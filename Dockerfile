FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y python3 netcat
COPY binaries/* /usr/local/bin/
COPY scripts /root/scripts
COPY topology /root/topology
WORKDIR /root
CMD /root/scripts/entrypoint.sh
