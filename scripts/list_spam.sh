#!/usr/bin/env sh
LL_HOSTNAME=http://localhost:8088

date

for i in $(seq -f %1.0f 1 5000000)
do
    echo "."
    curl ${LL_HOSTNAME}/list >> /dev/null
done

date