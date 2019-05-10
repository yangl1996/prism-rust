#!/bin/bash

echo "Updating apt cache"
sudo apt-get update -y

echo "Installing JDK"
sudo apt-get install openjdk-8-jdk -y
echo 'JAVA_HOME="/usr/lib/jvm/java-8-openjdk-amd64"' | sudo tee -a /etc/environment

echo "Installing jvmtop"
wget 'https://github.com/patric-r/jvmtop/releases/download/0.8.0/jvmtop-0.8.0.tar.gz' -O /home/ubuntu/download/jvmtop.tar.gz &>/dev/null
mkdir -p /home/ubuntu/download/jvmtop
tar xf /home/ubuntu/download/jvmtop.tar.gz -C /home/ubuntu/download/jvmtop &>/dev/null
sudo mv /home/ubuntu/download/jvmtop/jvmtop.jar /usr/local/bin
sudo mv /home/ubuntu/download/jvmtop/jvmtop.sh /usr/local/bin/jvmtop
sudo chmod +x /usr/local/bin/jvmtop
rm -rf /home/ubuntu/download/jvmtop

echo "Installing async-profiler"
wget 'https://github.com/jvm-profiling-tools/async-profiler/releases/download/v1.5/async-profiler-1.5-linux-x64.tar.gz' -O /home/ubuntu/download/async-profiler.tar.gz &>/dev/null
mkdir -p /home/ubuntu/download/async-profiler
tar xf /home/ubuntu/download/async-profiler.tar.gz -C /home/ubuntu/download/async-profiler &>/dev/null
sudo mv /home/ubuntu/download/async-profiler/build /usr/local/bin
sudo mv /home/ubuntu/download/async-profiler/profiler.sh /usr/local/bin/profiler
sudo chmod +x /usr/local/bin/profiler
rm -rf /home/ubuntu/download/async-profiler
echo 1 | sudo tee /proc/sys/kernel/perf_event_paranoid
echo 0 | sudo tee /proc/sys/kernel/kptr_restrict

echo "Adding sbt repository"
echo "deb https://dl.bintray.com/sbt/debian /" | sudo tee -a /etc/apt/sources.list.d/sbt.list
sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv 2EE0EA64E40A89B84B2DF73499E82A75642AC823
sudo apt-get update -y

echo "Installing sbt"
sudo apt-get install sbt -y

