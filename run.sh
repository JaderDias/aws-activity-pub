#!/bin/bash
docker-compose -f docker/test/docker-compose.yml kill \
&& docker-compose -f docker/test/docker-compose.yml up \
  --build \
  --detach \
&& cargo build --all-targets \
&& CUSTOM_DOMAIN=example.com \
DYNAMODB_TABLE=table_name \
LOCAL_DYNAMODB_URL=http://localhost:8000 \
REGION=eu-west-1 \
cargo run