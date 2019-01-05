# Spider Distributed Testbed

## Setting Up

1. Clone this repository to somewhere that you like
2. Add a line `Include config.d/spider` to the beginning of your `~/.ssh/config`
3. `mkdir -p ~/.ssh/config.d`. We will write EC2 instance info to `config.d/spider`
4. Install AWS CLI and configure your root key / IAM key
5. `cd` into `tools/`, and use `run.sh` to control the experiment

## `run.sh` Usage

Please refer to `./run.sh help`

## Typical Experiment Flow

```bash
./run.sh start-instances 3
# wait for several minutes for EC2 to start and auto-install docker, go, etc.
./run.sh init-docker
./run.sh build-images
./run.sh start-exp ../topology/some-topo-file.json experiment-name 120
# after the experiment finished
./run.sh stop-exp ../topology/some-topo-file.json
```

# Notes
^-p ^-q: dettach from the container

