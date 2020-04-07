#!/bin/sh

./start_net_mk1_hub.sh &
./start_daemon.sh ./provider.env &
./start_daemon.sh ./requestor.env &
