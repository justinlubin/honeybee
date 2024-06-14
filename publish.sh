#!/usr/bin/env bash

FROM=gui
TO=gh-pages-new

git clone git@github.com:justinlubin/honeybee.git \
  --branch gh-pages \
  --single-branch \
  gh-pages-old

mkdir $TO
mv gh-pages-old/.git $TO

for file in \
  CNAME \
  index.html \
  main.css \
  main.js \
  biology.hblib \
  biology.py
do
  cp $FROM/$file $TO/$file
done

for dir in \
  pkg
do
  cp -a $FROM/$dir $TO/$dir
done

cd $TO
git checkout gh-pages
git add * -f
git commit -m "Upload"
git push

cd ../
rm -rf gh-pages-old
rm -rf $TO
