.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	RAYON_NUM_THREADS=8 \
	cargo run -- benchmark \
	    -s ../benchmark/suites/ff/ \
	    -t 30000 \
	    -r 1 \
	    -q \
	    --algorithms E,EP,PBN_DLmem,PBN_DL \
	    > ../benchmark/data/data.tsv

.PHONY: full-benchmark
full-benchmark:
	cargo run -- benchmark \
	    -s ../benchmarks/suites/ff,../benchmark/suites/if/ \
	    -t 60000 \
	    -r 5 \
	    > ../benchmark/data/data.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
