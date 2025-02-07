#!/bin/bash

set -exv

cd ~/Projects/oc-mirror

#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --from file://test-mirror/ --destination docker://192.168.1.27:5000/test-mirror --skip-tls-verify 

bin/oc-mirror --config isc.yaml --from file://cicd docker://192.168.1.27:5000/cicd --v2 --dest-tls-verify=false

# for the error handling this format is important
echo -e "exit => $?"
