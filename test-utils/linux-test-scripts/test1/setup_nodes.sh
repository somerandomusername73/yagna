#!/bin/sh
# Setup provider and requestor nodes.

./start_net_mk1_hub.sh &

# Perform common node setup using provider environment variables
./setup_node.sh ./provider.env
./setup_node.sh ./requestor.env

# In future: perform other, possibly provider or requestor-specific
# steps (eg prepare data files)
