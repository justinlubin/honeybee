.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/generic/ \
	    -t 30000 \
	    -r 2 \
	    -q \
	    > ../benchmark/data/data.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
