#!/bin/sh

./create_app_key.sh ./provider.env
./create_app_key.sh ./requestor.env

./start_provider.sh &
./start_requestor.sh
