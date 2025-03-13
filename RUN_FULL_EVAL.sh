#!/usr/bin/env bash

################################################################################
## Set up variables and directories

MODE="full"

timestamp=$(date +'y%Y-m%m-d%d_hr%H-min%M-sec%S-ms%3N')

mkdir -p "benchmark/data/$MODE/$timestamp"
mkdir -p "benchmark/analysis/output/$MODE/$timestamp"

backend_dir=$(realpath "backend")
analysis_dir=$(realpath "benchmark/analysis")

data_dir=$(realpath "benchmark/data/$MODE/$timestamp")
output_dir=$(realpath "benchmark/analysis/output/$MODE/$timestamp")

################################################################################
## Benchmark synthesizers

cd $backend_dir

cargo build
hb="./target/debug/honeybee"

echo "Running part 1 of 3..."

$hb benchmark \
    --suite ../benchmark/suites/fin,../benchmark/suites/scal \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo,NaiveEnumeration,PrunedEnumeration \
    --replicates 5 \
    --timeout 120 \
    > "$data_dir/finscal.tsv"

echo "Running part 2 of 3..."

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo \
    --replicates 5 \
    --timeout 120 \
    > "$data_dir/inf-pbn.tsv"

echo "Running part 3 of 3..."

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms NaiveEnumeration,PrunedEnumeration \
    --replicates 5 \
    --timeout 120 \
    --limit 0 \
    > "$data_dir/inf-baseline.tsv"

################################################################################
## Combine results

cd $data_dir

cat \
    finscal.tsv \
    < (tail -n +2 inf-pbn.tsv) \
    < (tail -n +2 inf-baseline.txt) \
    > combined.tsv

################################################################################
## Generate graphs

echo "Analyzing results..."

cd $analysis_dir

uv run analyze.py 120 "$data_dir/combined.tsv" $output_dir

echo "All done!"
