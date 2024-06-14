#!/usr/bin/env bash

FROM=gui
TO=gh-pages-new

mkdir $TO
mv gh-pages/.git $TO

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

mv $TO gh-pages
cd gh-pages
git add -A
git commit -m "Upload"
git push
