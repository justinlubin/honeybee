#!/usr/bin/env bash

cd backend
echo "*** Building Honeybee... (you should see no errors)"
echo

cargo build

echo
echo "*** Checking that Honeybee can be built and run... (you should see a version string)"
echo

cargo run -- --version

echo
echo "*** Checking that uv is installed... (you should see a version string)"
echo

uv --version
