#!/bin/bash
set -e

docker build -f Dockerfile -t jscheel42/little-lookup:alpine --no-cache --pull .
docker tag jscheel42/little-lookup:alpine jscheel42/little-lookup:latest

if [ "$1" == "push" ]
then
    docker push jscheel42/little-lookup:alpine
    docker push jscheel42/little-lookup:latest
fi

if [[ -v BUILD_NUMBER ]]
then
    docker tag jscheel42/little-lookup:alpine jscheel42/little-lookup:$BUILD_NUMBER
    docker push jscheel42/little-lookup:$BUILD_NUMBER
fi
