#!/bin/bash

echo "Updating apt cache"
sudo apt-get update -y

echo "Installing JDK"
sudo apt-get install default-jdk -y

echo "Adding sbt repository"
echo "deb https://dl.bintray.com/sbt/debian /" | sudo tee -a /etc/apt/sources.list.d/sbt.list
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv 2EE0EA64E40A89B84B2DF73499E82A75642AC823
sudo apt-get update -y

echo "Installing sbt"
sudo apt-get install sbt -y

