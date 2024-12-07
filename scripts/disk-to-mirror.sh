#!/bin/bash

set -exvo pipefail

cd ~/Projects/rust-image-mirror

./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --from file://test-mirror/ --destination docker://192.168.1.27:5000/test-mirror --skip-tls-verify 

# for the error handling this format is important
echo -e "exit => $?"
