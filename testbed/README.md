# Prism Distributed Testbed

## Setting Up

1. Install jq
2. Install AWS CLI tool and configure the IAM Key and Region
3. Place the SSH key at `~/.ssh/prism.pem`
4. Place this line `Include config.d/prism` at the beginning of `~/.ssh/config`
5. Execute `mkdir -p ~/.ssh/config.d`
6. Start a local Ubuntu 18.04 VM that has Rust toolchain, `clang`, `build-essential` installed,
   is accessible by `ssh prism`, and can read the Github repository (preferably through SSH
   agent forwarding). We currently target Rust nightly.

## Usage

Run `./run.sh help` to view a list of available commands.

## Experiment Flow

1. `cd` to `testbed/`
2. Run `python3 scripts/generate_topo.py <NUM NODES> clique > clique.json`
3. Run `./run.sh build` to build the Prism binary
4. Run `./run.sh gen-payload clique.json` to generate the payload
5. Run `./run.sh sync-payload` to synchronize the payload to remote machines
6. Run `./run.sh run-exp <DURATION>` to run the experiment

## Log Files

instances.txt records the EC2 instances that are started in the following
format:

```
<Instance ID>,<Public IP>,<VPC IP>
```

nodes.txt records the Scorex nodes that are started, in the following format:

```
<Node Name>,<EC2 ID>,<Public IP>,<VPC IP>,<API IP>,<P2P IP>
```

## Algorand Experiment Flow

To run the additional Algorand experiments, start a local Ubuntu 18.04 VM with Algorand installed
and is accessible by `ssh algorand`. The Algorand binaries should live at ~/go/bin (the default
path for Golang binaries).
