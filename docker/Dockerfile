FROM ubuntu:latest as ubuntu

RUN apt-get update && apt-get install -y \
    apt-transport-https \
    clang-7 clang-tools-7 clang-7-doc \
    libclang-common-7-dev libclang-7-dev \
    libgflags-dev libsnappy-dev zlib1g-dev \
    libbz2-dev liblz4-dev libzstd-dev git curl

# Get Rocksdb
RUN git clone https://github.com/facebook/rocksdb.git 
WORKDIR rocksdb/
RUN make static_lib
WORKDIR /tmp/blockchains/prism-rust

# Get Rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH=/root/.cargo/bin:$PATH

