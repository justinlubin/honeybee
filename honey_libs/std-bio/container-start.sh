#!/usr/bin/env bash

# $1: environment folder

if [ ! -e "$1" ]; then
  cp environment $1
fi

uv run jupyter lab \
	--no-browser \
	--ip 0.0.0.0 \
	--port 8888 \
	--notebook-dir user-files \
	--allow-root
