#!/bin/bash
#
set -exv 

GIT_USER=$1
REPO=$2

echo -e "${GIT_USER} ${REPO}"

cd ~/Projects

rm -rf ${REPO}

git clone https://github.com/${GIT_USER}/${REPO}

cd $REPO

make build

# for the error handling this format is important
echo -e "exit => $?"
