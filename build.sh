#!/bin/bash
set -e

docker build -t jscheel42/little-lookup:latest --no-cache --pull .
if [ "$1" == "push" ]
then
    docker push jscheel42/little-lookup:latest
fi

if [[ -v BUILD_NUMBER ]]
then
    docker tag jscheel42/little-lookup:latest jscheel42/little-lookup:$BUILD_NUMBER
    docker push jscheel42/little-lookup:$BUILD_NUMBER
fi