.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	RAYON_NUM_THREADS=8 \
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 10000 \
	    -r 2 \
	    -q \
	    > ../benchmark/data/data.tsv

.PHONY: full-benchmark
quick-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 60000 \
	    -r 5 \
	    > ../benchmark/data/data.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
