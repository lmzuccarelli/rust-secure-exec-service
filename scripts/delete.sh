#!/bin/bash

set -exv

cd ~/Projects/oc-mirror

bin/oc-mirror --config delete-isc.yaml delete --delete-yaml-file cicd/working-dir/delete/delete-images.yaml  docker://192.168.1.27:5000/cicd --v2 --dest-tls-verify=false

# for the error handling this format is important
echo -e "exit => $?" 
