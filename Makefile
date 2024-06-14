.PHONY: wasm
wasm:
	wasm-pack build --target web --out-dir gui/pkg

.PHONY: publish
publish:
	git subtree push --prefix gui origin gh-pages
