#!/usr/bin/env bash

echo "Checking that uv is installed... (you should see a version string)"

uv --version

echo
echo "Checking that Honeybee can be built and run... (you should see a version string)"

cd backend
cargo run -- --version
