.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir gui/pkg
