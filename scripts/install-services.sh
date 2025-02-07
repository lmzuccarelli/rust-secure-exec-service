#!/bin/bash
#
set -exv 

MICROSERVICE=$1

cd ~/Projects/microservices

tee ${MICROSERVICE}.service <<EOF
[Service]
ExecStart=/home/lzuccarelli/Projects/microservices/${MICROSERVICE} /home/lzuccarelli/Projects/microservices
EOF

sudo cp ${MICROSERVICE}.service /etc/systemd/system/
sudo chmod 755 /etc/systemd/system/${MICROSERVICE}.service

sudo systemctl start ${MICROSERVICE}

# for the error handling this format is important
echo -e "exit => $?"
