.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	RAYON_NUM_THREADS=24 \
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 10000 \
	    -r 2 \
	    -q \
	    --tasks Particular \
	    > ../benchmark/data/data.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
