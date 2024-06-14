.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir gui/pkg

.PHONY: publish
publish:
	./publish.sh
