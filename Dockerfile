FROM ubuntu:18.04

RUN apt-get update
RUN apt-get install -y python3 netcat
COPY binaries/* /usr/local/bin/
COPY assets /root/scripts
WORKDIR /root/scripts
CMD /bin/bash
#CMD /root/bootstrap/init.sh
