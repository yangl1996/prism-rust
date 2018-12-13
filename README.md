# How to start a cluster of docker hosts

1. `docker swarm init` on one machine (this host will become the manager of the overlay network), and make a note of the printed text.
2. On the other machines, join the newly-created overlay network following the instruction.
3. On the manager machine, create a new docker overlay network specifically for Spider `sudo docker network create -d overlay --attachable spider`.
4. On each machine, start some docker containers `sudo docker run -it --name spider01 --network spider test /bin/bash`. Note that on non-manager machines, the overlay network "spider" will not be there at the beginning. But once the container is created, it will automatically be created (and have the same network ID as the manager machine).
5. Use `docker inspect spider01` to see the IP of the containers. They can ping each other now. Or, run `nc -l -p 12345` on one machine, and run `nc $ip 12345` on the other, and verify that they can communicate.

# Notes
^-p ^-q: dettach from the container

