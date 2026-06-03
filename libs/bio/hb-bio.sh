#!/usr/bin/env bash

set -e

HONEYBEE_RELEASE=8

ARCH="$1"

if [ -z "${ARCH}" ]; then
  case $(uname -m) in
      x86_64) ARCH="amd64" ;;
      arm64) ARCH="arm64" ;;
  esac
fi

if [ -z "${ARCH}" ]; then
  echo "error: could not automatically determine architecture"
  echo "Please re-run with first argument as either 'amd64' or 'arm64'"
fi

IMAGE_NAME=ghcr.io/justinlubin/hb-bio:${HONEYBEE_RELEASE}-${ARCH}

CMD=""

if [[ "$2" == "local" ]]; then
  echo "Using 'docker' for local image..."
  CMD="docker"
else
  echo "Using 'podman' for remote image..."
  podman machine info 2>/dev/null \
    | grep -q "machinestate: Running" \
    || podman machine start
  podman pull ${IMAGE_NAME}
  CMD="podman"
fi

EXTRAS=""
if [[ "$3" == "interactive" ]]; then
  EXTRAS="--entrypoint /usr/bin/bash"
fi

mkdir -p user-files

set -x

$CMD run -it \
  -p 8888:8888 \
  --mount type=bind,source="$(pwd)/user-files",target="/root/user-files" \
  $EXTRAS \
  $IMAGE_NAME
