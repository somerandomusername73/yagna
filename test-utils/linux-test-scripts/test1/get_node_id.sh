#!/bin/sh
# Get node id from the yagna daemon

ENV_FILE=$1
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE" >> /dev/stderr; exit 1; }

# Note: this may easily break if the output format of `yagna id show` changes
./yagnacli.sh "$ENV_FILE" id show \
    | grep -Po '(?<=nodeId: )0x[0-9a-f]{40}'
