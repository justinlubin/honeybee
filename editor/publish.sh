#!/usr/bin/env bash

make clean
make all

shopt -s extglob

TO="__gh-pages"

git clone git@github.com:justinlubin/honeybee.git \
  --branch gh-pages \
  --single-branch \
  "$TO"

cd "$TO"
git checkout gh-pages
rm -rf !(.git)

cd ../

cp -r www/. $TO

cd $TO
git add * -f
git commit -m "Upload"
git push

cd ../
rm -rf $TO
