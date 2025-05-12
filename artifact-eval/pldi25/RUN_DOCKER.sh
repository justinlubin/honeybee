#!/usr/bin/env bash

source_data="$(pwd)"/DOCKER_MOUNT/data
source_output="$(pwd)"/DOCKER_MOUNT/output

mkdir -p $source_data
mkdir -p $source_output

target_data=/home/ubuntu/benchmark/data
target_output=/home/ubuntu/benchmark/analysis/output

docker run \
    -i -t \
    --mount type=bind,source="$source_data",target="$target_data" \
    --mount type=bind,source="$source_output",target="$target_output" \
    pldi25ae-honeybee
