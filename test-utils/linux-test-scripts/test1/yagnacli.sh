#!/bin/sh

ENV_FILE=$1
[ -n "$ENV_FILE" ] || { echo "Usage: $0 ENV_FILE" >> /dev/stderr; exit 1; }

. "$ENV_FILE"

export YAGNA_API_URL
export CENTRAL_NET_HOST
export GSB_URL
cargo run --bin yagna -- $2 $3 $4 $5 $6 $7
