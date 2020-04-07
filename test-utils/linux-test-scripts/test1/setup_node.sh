#!/bin/sh
# This script (re)creates the data directory used for the yagna daemon
# and runs the daemon until it generates default node identity.
#
# In future yagna deb package installation may be also performed here.
#
# Network hub must be running on $CENTRAL_NET_HOST, or else the requestor
# daemon will hang on exit.

ENV_FILE="$1"
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE"; >> /dev/stderr; exit 1; }

# Read environment file to set DATA_DIR
. "$ENV_FILE"

# Create data directory
[ ! -e "$DATA_DIR" ] || rm -r "$DATA_DIR"
mkdir "$DATA_DIR"

# Note: this relies on the yagna daemon printing a specific message
sh ./start_daemon.sh "$ENV_FILE" 2>&1 \
    | tee /dev/stderr \
    | grep -m1 'using default identity: '
