USERHOME=/home/ubuntu
EXPROOT="${USERHOME}/spider-docker"
export PATH="$PATH:/snap/bin"

git clone https://github.com/yangl1996/spider-docker.git $EXPROOT
cd $EXPROOT
bash build.sh
docker build -t spider .

