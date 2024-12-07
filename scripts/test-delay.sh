#!/bin/bash

#set -exv

for i in $(seq 1 5); 
  do
    echo "hello $i";
    sleep 3; 
done

echo -e "exit => $?"

