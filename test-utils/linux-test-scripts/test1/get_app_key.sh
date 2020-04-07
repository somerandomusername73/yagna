#!/bin/sh

ENV_FILE=$1
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE" >> /dev/stderr; exit 1; }

# Read KEY_NAME from node env vars
. "$ENV_FILE"

# Note: this depends on the specific output format from `yagna app-key list`
./yagnacli.sh "$ENV_FILE" app-key list \
    | grep "$KEY_NAME" \
    | sed -e 's/\s*│\s*/ /g' -e 's/^ //' \
    | cut -d ' ' -f2
