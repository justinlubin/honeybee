#!/usr/bin/bash

if [ ! -e "$HONEYBEE_USER_ENVIRONMENT" ]; then
  cp -r environment $HONEYBEE_USER_ENVIRONMENT
fi

echo
echo "    >>> Success!! <<<"
echo
echo "You've loaded:"
echo
echo "    Honeybee Biology Container, Release ${HONEYBEE_RELEASE} (${HONEYBEE_COMMIT})"
echo

uv run jupyter lab \
  --log-level=CRITICAL \
	--no-browser \
	--ip 0.0.0.0 \
	--port 8888 \
	--notebook-dir user-files \
	--allow-root
