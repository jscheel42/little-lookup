#!/usr/bin/env sh
LL_HOSTNAME=https://little-lookup-945130679167.us-central1.run.app
# LL_HOSTNAME=http://localhost:8088

date

while true
do
    echo "."
    curl ${LL_HOSTNAME}/list >> /dev/null
done

date
