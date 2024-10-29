.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	RAYON_NUM_THREADS=16 \
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 7000 \
	    -r 1 \
	    -q \
	    > ../benchmark/data/data.tsv

.PHONY: full-benchmark
full-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 60000 \
	    -r 5 \
	    > ../benchmark/data/data.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
