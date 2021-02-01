#!/usr/bin/env sh
LL_HOSTNAME=http://localhost:8088

date

for i in $(seq -f %1.0f 1000001 2000000)
do
    curl ${LL_HOSTNAME}/update/key${i}/var${i}
done

date