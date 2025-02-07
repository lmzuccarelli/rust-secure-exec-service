#!/bin/bash

set -exv

cd ~/Projects/oc-mirror

#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror --skip-manifest-check release
#./target/release/image-mirror --config examples/imagesetconfig.yaml --loglevel info --destination file://test-mirror 

cat <<EOF >>delete-isc.yaml
kind: DeleteImageSetConfiguration
apiVersion: mirror.openshift.io/v2alpha1
delete:
  platform:
    graph: true 
    architectures:
    - amd64
    channels:
    - name: stable-4.16
      minVersion: '4.16.1'
      maxVersion: '4.16.1'
      shortestPath: true
  helm:
    repositories:
      - name: sbo
        url: https://redhat-developer.github.io/service-binding-operator-helm-chart/
  additionalImages:
  - name: registry.redhat.io/ubi8/ubi:latest
  - name: quay.io/openshifttest/hello-openshift@sha256:61b8f5e1a3b5dbd9e2c35fd448dc5106337d7a299873dd3a6f0cd8d4891ecc27
  operators:
  - catalog: registry.redhat.io/redhat/redhat-operator-index@sha256:a44ae9bb5ff9a4a988a8bf219011c463c4f9e764f0d9aced17b2d399729ebd4a
    packages:
    - name: devworkspace-operator
EOF

bin/oc-mirror --config delete-isc.yaml delete --generate --workspace file://cicd docker://192.168.1.27:5000/cicd --v2

# for the error handling this format is important
echo -e "exit => $?"

