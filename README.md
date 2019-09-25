# Prism

Rust implementation of the Prism consensus protocol.

## Paper

__Prism: Scaling Bitcoin by 10,000x__

_Lei Yang (MIT CSAIL), Vivek Bagaria (Stanford University), Gerui Wang (UIUC), Mohammad Alizadeh (MIT CSAIL), David Tse (Stanford University), Giulia Fanti (CMU), Pramod Viswanath (UIUC)_

Abstract: Bitcoin is the first fully decentralized permissionless blockchain protocol and achieves a high level of security: the ledger it maintains has guaranteed liveness and consistency properties as long as the adversary has less compute power than the honest nodes. However, its throughput is only 7 transactions per second and the confirmation latency can be up to hours. Prism is a new blockchain protocol which is designed to achieve a natural scaling of Bitcoin's performance while maintaining its full security guarantees. We present an implementation of Prism which achieves a throughput of 70,000 transactions per second and confirmation latencies of tens of seconds.

## Build

This project requires Rust `nightly`. To build the binary, run `cargo build --release`.

The first build could take several mintues, mostly due to building RocksDB.

## Testbed

The scripts used in the evaluation section of the paper are located in `/testbed`.

