#!/bin/bash

set -exv

cd ~/Projects/rust-image-mirror

./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror --skip-manifest-check release
#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror 

# for the error handling this format is important
echo -e "exit => $?"


