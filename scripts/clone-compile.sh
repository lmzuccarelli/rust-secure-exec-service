#!/bin/bash
#
set -exv 

cd ~/Projects

rm -rf rust-image-mirror

git clone https://github.com/lmzuccarelli/rust-image-mirror

cd rust-image-mirror

make build

# for the error handling this format is important
echo -e "exit => $?"
