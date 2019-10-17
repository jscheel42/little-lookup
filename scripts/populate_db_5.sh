#!/usr/bin/env sh
LL_HOSTNAME=http://kube01.homenet:30008

date

for i in $(seq 4000001 5000000)
do
    curl ${LL_HOSTNAME}/item/key${i}/var${i}
done

date