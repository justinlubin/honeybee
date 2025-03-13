#!/usr/bin/env bash

# Call this script with one argument: the number of threads to use (default 2)

################################################################################
## Set up variables and directories

MODE="quick"

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

echo "Benchmarking with ${1:-2} threads (configurable via first argument)"

export RAYON_NUM_THREADS=${1:-2}

$hb benchmark \
    --suite ../benchmark/suites/fin,../benchmark/suites/scal \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo,NaiveEnumeration,PrunedEnumeration \
    --replicates 1 \
    --timeout 120 \
    --limit 1 \
    --parallel \
    > "$data_dir/finscal.tsv"

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo \
    --replicates 1 \
    --timeout 120 \
    --limit 1 \
    --parallel \
    > "$data_dir/inf-pbn.tsv"

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms NaiveEnumeration,PrunedEnumeration \
    --replicates 1 \
    --timeout 120 \
    --limit 0 \
    --parallel \
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

cd $analysis_dir

uv run analyze.py 120 "$data_dir/combined.tsv" $output_dir
