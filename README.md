# Build containers

1. `bash build.sh`. This script builds/downloads all required binaries and moves them to `binaries/`.
2. `docker build -t spider .`. It moves all binaries to the containers and installs python3 and netcat in the container.

# Start container into `bash`

Run `docker run -it spider /bin/bash` to start into bash.

# Set up experiment

1. `docker swarm init` on one machine (this host will become the manager of the overlay network), and make a note of the printed text.
2. On the other machines, join the newly-created overlay network following the instruction.
3. On the manager machine, create a new docker overlay network specifically for Spider `sudo docker network create -d overlay --subnet 10.0.1.0/16 --attachable spider`.

# Notes
^-p ^-q: dettach from the container

