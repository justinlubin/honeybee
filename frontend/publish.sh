#!/usr/bin/env bash

cd ../

shopt -s extglob

FROM=frontend
TO=gh-pages

git clone git@github.com:justinlubin/honeybee.git \
  --branch gh-pages \
  --single-branch \
  $TO

cd $TO
git checkout gh-pages
rm -rf !(.git)

cd ../$FROM

cp -a -L \
  !(publish.sh|biome.json|package.json|package-lock.json|node_modules) \
  ../$TO

cd ../$TO
git add -A
git add * -f
git commit -m "Upload"
git push

cd ../
rm -rf $TO
