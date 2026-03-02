#!/bin/bash

TAG="latest"
FEATURES=""

while getopts "t:f:" opt; do
    case $opt in
        t) TAG="$OPTARG" ;;
        f) FEATURES="$OPTARG" ;;
        *) echo "invalid option ($opt)"; exit 1 ;;
    esac
done

docker build --cpu-period=100000 --cpu-quota=50000 -t myblog:$TAG --build-arg FEATURES="$FEATURES" .
