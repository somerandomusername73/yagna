#!/bin/sh

# Read env vars
. ./requestor.env

NODE_ID=$(./get_node_id.sh ./requestor.env)
APP_KEY=$(./get_app_key.sh ./requestor.env)

curl -X POST "http://localhost:5001/admin/import-key" \
     -H "accept: application/json" \
     -H "Content-Type: application/json-patch+json" \
     -d "[ { \"key\": \"${APP_KEY}\", \"nodeId\": \"${NODE_ID}\" }]"

export RUST_LOG=info
cargo run --bin ya-requestor -- \
      --activity-url "$YAGNA_ACTIVITY_URL" \
      --app-key "$APP_KEY" \
      --market-url "$YAGNA_MARKET_URL"

