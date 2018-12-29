FROM ubuntu:18.04

RUN apt-get update && apt-get -y upgrade
RUN apt-get install git software-properties-common -y
RUN add-apt-repository ppa:longsleep/golang-backports
RUN apt-get update
RUN apt-get install golang-go -y
ENV GOPATH "/root/gocode"
ENV PATH="${PATH}:${GOPATH}/bin"
RUN go get -u github.com/golang/dep/cmd/dep
RUN go get -d github.com/urfave/cli
RUN go get -d github.com/vibhaa/lnd
RUN rm -rf $GOPATH/src/github.com/lightningnetwork/lnd
RUN mv $GOPATH/src/github.com/vibhaa/lnd $GOPATH/src/github.com/lightningnetwork
WORKDIR $GOPATH/src/github.com/lightningnetwork/lnd
RUN git checkout stats
RUN git pull origin stats
RUN make && make install
RUN make btcd
RUN apt-get install -y etcd netcat
RUN apt-get install -y jq
COPY bootstrap /root/bootstrap
RUN cp -r /root/bootstrap/payment /root/gocode/src/github.com/lightningnetwork/lnd
WORKDIR /root/gocode/src/github.com/lightningnetwork/lnd/payment
RUN go get -d go.etcd.io/etcd/client
RUN go build
RUN cp payment /root/bootstrap/run
WORKDIR /root/bootstrap
CMD /root/bootstrap/init.sh
