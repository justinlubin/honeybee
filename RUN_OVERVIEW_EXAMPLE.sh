#!/usr/bin/env bash

cd backend
cargo run -- interact -l ../benchmark/suites/fin/_suite.hblib.toml -p ../benchmark/suites/fin/bio_rna_seq.hb.toml
