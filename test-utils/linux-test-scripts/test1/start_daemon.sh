#!/bin/sh
# Start a yagna daemon

ENV_FILE=$1
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE" >> /dev/stderr; exit 1; }

# Read node configuration
. "$ENV_FILE"

export CENTRAL_NET_HOST
export GSB_URL
export YAGNA_MARKET_URL
export YAGNA_API_URL
export YAGNA_ACTIVITY

cargo run --bin yagna service run -d "$DATA_DIR"
