# Prism Distributed Testbed

This directory holds the scripts for running experiments and reproducing the results in the paper.

For a quick demo on how to reproduce out results, please check out this [screen recording](https://asciinema.org/a/YGz4dIkfKz4DrHLtVIGSfpmly).

## Setting Up

### Set Up AWS Account

1. Configure an IAM role with the following permissions
    - DescribeInstances
    - DescribeInstanceStatus
    - CreateTags
    - RunInstances
    - TerminateInstances
2. Create an EC2 key pair
3. Create an EC2 security group that allows all traffic to go in/out
4. Create an EC2 Launch Template with the following configurations
    - AMI: Ubuntu 18.04
    - Instance type: `c5d.4xlarge`
    - Key pair: the one just created
    - Network type: VPC
    - Security Groups: the one just created
    - Storage (Volumes): 32 GiB `gp2` volume, delete on termination
    - Instance tags: Key=prism, Value=distributed-testing, tag instance
5. Create a S3 bucket with name `prism-binary` and set it to be publicly accessible by putting the following in the bucket policy

```json
{
    "Version": "2008-10-17",
    "Statement": [
        {
            "Sid": "AddPerm",
            "Effect": "Allow",
            "Principal": {
                "AWS": "*"
            },
            "Action": "s3:GetObject",
            "Resource": "arn:aws:s3:::prism-binary/*"
        }
    ]
}
```

### Install Dependencies

1. Modify `run.sh` to use the Launch Tempate ID of the one just created
2. Place the SSH key just created at `~/.ssh/prism.pem`
3. Place this line `Include config.d/prism` at the beginning of `~/.ssh/config`
4. Execute `mkdir -p ~/.ssh/config.d`
5. Install `jq` and Golang
6. Install AWS CLI tool and configure the IAM Key to be the one just created, and Region to be the closest one
7. Start a local Ubuntu 18.04 VM that has Rust toolchain, `clang`, `build-essential` installed,
   is accessible by `ssh prism`, and can read the Github repository (preferably through SSH
   agent forwarding), so that it can compile the binary
8. Build the telematics tool by `cd telematics && go build`

## Usage

Run `./run.sh help` to view a list of available commands.

## Experiment Flow

1. `cd` to `testbed/`
2. Run `python3 scripts/generate_topo.py <NUM NODES> randreg <DEGREE> > randreg.json`
3. Run `./run.sh build` to build the Prism binary
4. Run `./run.sh start-instances 100` to start 100 instances
5. Run `./run.sh tune-tcp`, `./run.sh shape-traffic 120 400000`, `./run.sh mount-nvme` to configure the servers
4. Run `./run.sh gen-payload randreg.json` to generate the payload
5. Run `./run.sh sync-payload` to push the test files to remote servers
6. Run `./run.sh run-exp` to run the experiment
7. Run `telematics/telematics log` to monitor the performance
8. To stop the instances, run `./run.sh stop-instances`

## Log Files

`instances.txt` records the EC2 instances that are started in the following
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
