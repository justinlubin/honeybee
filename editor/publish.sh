#!/usr/bin/env bash

make clean
make all

shopt -s extglob

TO="__gh-pages"
UNSTABLE="unstable"

git clone git@github.com:justinlubin/honeybee.git \
  --branch gh-pages \
  --single-branch \
  "$TO"

cd "$TO"
git checkout gh-pages

if [ "$1" == "new-version" ]; then
    rm -rf !(.git)
    cp -r ../www/. .
fi

mkdir -p "$UNSTABLE"
rm -rf "$UNSTABLE"
mkdir "$UNSTABLE"
cp -r ../www/. "$UNSTABLE"

git add * -f
git commit -m "Upload"
git push

cd ../
rm -rf "$TO"
