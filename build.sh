#!/bin/bash
set -e

docker build -f Dockerfile-glibc -t jscheel42/little-lookup:glibc --no-cache --pull .
docker build -f Dockerfile-musl -t jscheel42/little-lookup:musl --no-cache --pull .
docker tag jscheel42/little-lookup:glibc jscheel42/little-lookup:latest

if [ "$1" == "push" ]
then
    docker push jscheel42/little-lookup:glibc
    docker push jscheel42/little-lookup:musl
    docker push jscheel42/little-lookup:latest
fi

if [[ -v BUILD_NUMBER ]]
then
    docker tag jscheel42/little-lookup:glibc jscheel42/little-lookup:glibc-$BUILD_NUMBER
    docker push jscheel42/little-lookup:glibc-$BUILD_NUMBER

    docker tag jscheel42/little-lookup:musl jscheel42/little-lookup:musl-$BUILD_NUMBER
    docker push jscheel42/little-lookup:musl-$BUILD_NUMBER
fi
