#!/bin/sh
# Creates an app key. Drops the key first if it already exists.

ENV_FILE=$1
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE" >> /dev/stderr; exit 1; }

# Read KEY_NAME from node env vars
. "$ENV_FILE"

./yagnacli.sh "$ENV_FILE" app-key drop "$KEY_NAME"
./yagnacli.sh "$ENV_FILE" app-key create "$KEY_NAME"
