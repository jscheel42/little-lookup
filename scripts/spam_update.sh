#!/usr/bin/env sh
LL_HOSTNAME=https://little-lookup-945130679167.us-central1.run.app
# LL_HOSTNAME=http://localhost:8088

date

while true
do

    for i in $(seq -f %1.0f 1 5000000)
    do
        curl ${LL_HOSTNAME}/update/key${i}/var${i}?psk=$LITTLE_LOOKUP_PSK_WRITE_TEST_CD
    done

done

date
