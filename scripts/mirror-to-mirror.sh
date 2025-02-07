#!/bin/bash

set -exv

cd ~/Projects/oc-mirror

#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror --skip-manifest-check release
#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror 

cat <<EOF >>isc-m2m.yaml
kind: ImageSetConfiguration
apiVersion: mirror.openshift.io/v2alpha1
mirror:
  additionalImages:
  - name: registry.redhat.io/ubi9/ubi:latest
EOF

bin/oc-mirror --config isc-m2m.yaml --workspace file://cicd docker://192.168.1.27:5000/cicd --v2 --dest-tls-verify=false

# for the error handling this format is important
echo -e "exit => $?"

