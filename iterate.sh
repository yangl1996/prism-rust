#!/bin/bash
sudo docker build -t test .
sudo docker run -it test /bin/bash
