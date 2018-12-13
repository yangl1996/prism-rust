# How to start a cluster of docker hosts

1. `docker swarm init` on one machine (this host will become the manager of the overlay network), and make a note of the printed text.
2. On the other machines, join the newly-created overlay network following the instruction.
3. On the manager machine, create a new docker overlay network specifically for Spider `sudo docker network create -d overlay --attachable spider`.

# How to start containers

`sudo docker run -it --name spider1 -e NODENAME=spider1 --network spider --ip 10.0.1.100 test /bin/bash`

# Containers can be addressed by its IP or container name

For example, `ping spider01`.

# How to start btcd

1. Put `btcd.conf` at `/root/.btcd/btcd.conf`
2. `btcd`

# Notes
^-p ^-q: dettach from the container

# Problems

All containers must be present on the network before btcd can be started - otherwise, btcd will complain about can't lookup names.
