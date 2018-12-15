# How to start a cluster of docker hosts

1. `docker swarm init` on one machine (this host will become the manager of the overlay network), and make a note of the printed text.
2. On the other machines, join the newly-created overlay network following the instruction.
3. On the manager machine, create a new docker overlay network specifically for Spider `sudo docker network create -d overlay --subnet 10.0.0.0/16 --attachable spider`.

# Notes
^-p ^-q: dettach from the container
`lncli -n simnet`

