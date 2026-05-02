#!/usr/bin/env bash

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
    cd ../
    make clean
    PUBLISH_STABLE=true make all
    cd "$TO"
    cp -r ../www/. .
fi

mkdir -p "$UNSTABLE"
rm -rf "$UNSTABLE"
mkdir "$UNSTABLE"

cd ../
make clean
make all
cd "$TO"

cp -r ../www/. "$UNSTABLE"

git add * -f
git commit -m "Upload"
git push

cd ../
rm -rf "$TO"
