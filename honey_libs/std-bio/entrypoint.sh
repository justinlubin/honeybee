#!/usr/bin/bash

echo
echo "################################################################################"
echo "## Starting Honeybee Standard Bio Docker Container (${HONEYBEE_VERSION})"
echo "################################################################################"
echo

echo "User environment: $HONEYBEE_ENVIRONMENT"
echo

if [ ! -e "$1" ]; then
  cp -r environment $HONEYBEE_ENVIRONMENT
fi

uv run jupyter lab \
	--no-browser \
	--ip 0.0.0.0 \
	--port 8888 \
	--notebook-dir user-files \
	--allow-root
