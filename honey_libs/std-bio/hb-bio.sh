#!/usr/bin/env bash

set -e
set -x

HB_VERSION=0.6.1

if [[ "$1" != "amd64" && "$1" != "arm64" ]]; then
  echo "Usage: $(basename "$0") <arch>" >&2
  echo >&2
  echo "  arch  Target architecture. Must be one of:" >&2
  echo "          amd64    x86-64 (e.g. Intel/AMD)" >&2
  echo "          arm64    AArch64 (e.g. Apple Silicon)" >&2
  echo >&2
  echo "Examples:" >&2
  echo "  $(basename "$0") amd64" >&2
  echo "  $(basename "$0") arm64" >&2
  exit 1
fi

CMD=""
IMAGE_NAME=ghcr.io/justinlubin/hb-bio:${HB_VERSION}-$1

if [[ "$2" == "local" ]]; then
  echo "Using 'docker' for local image..."
  CMD="docker"
else
  echo "Using 'podman' for remote image..."
  podman machine info 2>/dev/null \
    | grep -q "machinestate: Running" \
    || podman machine start
  podman pull ghcr.io/justinlubin/hb-bio:${HB_VERSION}-$1
  CMD="podman"
fi

EXTRAS=""
if [[ "$3" == "interactive" ]]; then
  EXTRAS="--entrypoint /usr/bin/bash"
fi

mkdir -p user-files

$CMD run -it \
  -p 8888:8888 \
  --mount type=bind,source="$(pwd)/user-files",target="/root/user-files" \
  $EXTRAS \
  $IMAGE_NAME
