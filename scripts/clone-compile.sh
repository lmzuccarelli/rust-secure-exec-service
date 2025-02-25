#!/bin/bash
#
set -exv 

GIT_USER=$1
REPO=$2
BRANCH="${3:-main}"
CLEAN="${4:-false}"

echo -e "${GIT_USER} ${REPO}"

cd ~/Projects

if [ "${CLEAN}" == "true" ];
then
  rm -rf ${REPO}
fi

if [ ! -d ${REPO} ];
then
  git clone -b ${BRANCH} https://github.com/${GIT_USER}/${REPO}
fi

cd $REPO

make build

# for the error handling this format is important
echo -e "exit => $?"
