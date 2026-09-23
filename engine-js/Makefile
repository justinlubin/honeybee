pkg: $(wildcard ../engine/src/*.rs) $(wildcard src/*.rs)
	cargo build --lib --target wasm32-unknown-unknown --release && \
	wasm-bindgen target/wasm32-unknown-unknown/release/honeybee_js.wasm \
		--out-dir pkg \
		--target web
