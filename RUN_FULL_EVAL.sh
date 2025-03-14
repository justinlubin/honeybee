#!/usr/bin/env bash

echo -n "The current time is: "
date +"%D %T"

################################################################################
# %% Set up variables and directories

MODE="full"

timestamp=$(date +'%Y-%m-%d_%H-%M-%S')

mkdir -p "benchmark/data/$MODE/$timestamp"
mkdir -p "benchmark/analysis/output/$MODE/$timestamp"

backend_dir=$(realpath "backend")
analysis_dir=$(realpath "benchmark/analysis")

data_dir=$(realpath "benchmark/data/$MODE/$timestamp")
output_dir=$(realpath "benchmark/analysis/output/$MODE/$timestamp")

echo "Using timestamp '$timestamp'"

################################################################################
# %% Benchmark synthesizers

cd $backend_dir

cargo build
hb="./target/debug/honeybee"

echo "Running part 1 of 3..."

echo -n "The current time is: "
date +"%D %T"

$hb benchmark \
    --suite ../benchmark/suites/fin,../benchmark/suites/scal \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo,NaiveEnumeration,PrunedEnumeration \
    --replicates 5 \
    --timeout 120 \
    > "$data_dir/finscal.tsv"

echo "Running part 2 of 3..."

echo -n "The current time is: "
date +"%D %T"

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms PBNHoneybee,PBNHoneybeeNoMemo \
    --replicates 5 \
    --timeout 120 \
    > "$data_dir/inf-pbn.tsv"

echo "Running part 3 of 3..."

echo -n "The current time is: "
date +"%D %T"

$hb benchmark \
    --suite ../benchmark/suites/inf \
    --algorithms NaiveEnumeration,PrunedEnumeration \
    --replicates 5 \
    --timeout 120 \
    --limit 0 \
    > "$data_dir/inf-baseline.tsv"

################################################################################
# %% Combine results

cd $data_dir

cat \
    finscal.tsv \
    <(tail -n +2 inf-pbn.tsv) \
    <(tail -n +2 inf-baseline.tsv) \
    > combined.tsv

################################################################################
# %% Generate graphs

echo "Analyzing results..."

cd $analysis_dir

uv run analyze.py 120 "$data_dir/combined.tsv" $output_dir

echo "All done!"

echo -n "The current time is: "
date +"%D %T"
