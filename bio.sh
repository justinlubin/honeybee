#!/usr/bin/env bash
cargo run \
  gui/hblib/biology.hblib \
  gui/hblib/biology.py \
  examples/$1.hb \
  output/$1.ipynb
