USERHOME=/home/ubuntu
EXPROOT="${USERHOME}/spider-docker"

git clone https://github.com/yangl1996/spider-docker.git $EXPROOT
cd $EXPROOT
wget 'https://github.com/yangl1996/spider-docker/releases/download/v0.1/binaries.tar.gz'
tar xf binaries.tar.gz
rm binaries.tar.gz
docker build -t spider .

