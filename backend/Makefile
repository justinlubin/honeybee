.PHONY: build
build:
	cargo build

.PHONY: quick-benchmark
quick-benchmark:
	RAYON_NUM_THREADS=7 \
	cargo run -- benchmark \
	    -s ../benchmark/suites/fin,../benchmark/suites/inf,../benchmark/suites/scal/ \
	    -t 10000 \
	    -r 1 \
	    -q \
	    > ../benchmark/data/data.tsv

.PHONY: fin-benchmark
fin-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/fin/ \
	    -t 120000 \
	    -r 5 \
	    > ../benchmark/data/fin.tsv

.PHONY: inf-benchmark
fin-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/inf/ \
	    -t 120000 \
	    -r 5 \
	    -a PBN_DL:Particular,PBN_DLmem:Particular \
	    > ../benchmark/data/inf.tsv

.PHONY: scal-benchmark
fin-benchmark:
	cargo run -- benchmark \
	    -s ../benchmark/suites/scal/ \
	    -t 120000 \
	    -r 5 \
	    -a E:All,EP:All,PBN_DLmem:Particular \
	    > ../benchmark/data/inf.tsv

.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir ../frontend/pkg
