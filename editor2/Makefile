# Requirements:
#   cargo
#   gleam
#   uv
#   wasm-bindgen
#     cargo install -f wasm-bindgen-cli --version 0.2.108
#   wasm32-unknown-unknown Rust target
#     rustup target add wasm32-unknown-unknown

HERE_DIR := editor2
JS_NAME := editor2.js
ENGINE_DIR := ../engine
LANG_DIR := ../honey
GLUE_DIR := ../libs/bio
OUT_DIR := www
CURRENT_COMMIT := $(shell git rev-parse --short HEAD)

.PHONY: all
all: \
	$(OUT_DIR)/$(JS_NAME) \
	$(OUT_DIR)/pkg/honeybee.js \
	$(OUT_DIR)/bio.hblib.toml \
	$(OUT_DIR)/launch-notebook.sh

.PHONY: clean
clean:
	rm -f $(OUT_DIR)/elm.js
	rm -f $(OUT_DIR)/pkg/honeybee.js
	rm -f $(OUT_DIR)/bio.hblib.toml
	rm -f $(OUT_DIR)/launch-notebook.sh

# Depends on honeybee.js to ensure versions are in-sync
$(OUT_DIR)/$(JS_NAME): $(wildcard src/*.gleam) $(OUT_DIR)/pkg/honeybee.js
	gleam run -m lustre/dev build --no-html --outdir=www || touch $(OUT_DIR)/$(JS_NAME)
	sed "s/<<<COMMIT-SHORT-HASH>>>/$(CURRENT_COMMIT)/g" $(OUT_DIR)/$(JS_NAME) \
	| if [ -n "${PUBLISH_STABLE}" ]; then sed "s/<<<UNSTABLE-INDICATOR>>>/STABLE/g"; else sed "s/<<<UNSTABLE-INDICATOR>>>/UNSTABLE/g"; fi \
	> $(OUT_DIR)/$(JS_NAME).tmp
	mv $(OUT_DIR)/$(JS_NAME).tmp $(OUT_DIR)/$(JS_NAME)

$(OUT_DIR)/pkg/honeybee.js: $(wildcard $(ENGINE_DIR)/src/*.rs)
	cd $(ENGINE_DIR) && \
	cargo build --lib --target wasm32-unknown-unknown --release && \
	wasm-bindgen target/wasm32-unknown-unknown/release/honeybee.wasm \
		--out-dir ../$(HERE_DIR)/$(OUT_DIR)/pkg \
		--target web

$(OUT_DIR)/bio.hblib.toml: $(wildcard $(LANG_DIR)/*.py) $(wildcard $(GLUE_DIR)/*.py)
	cd $(GLUE_DIR) && ty check
	uv run \
		--project $(GLUE_DIR) \
		$(GLUE_DIR)/bio.py \
		> $(OUT_DIR)/bio.hblib.toml && \
	cd $(ENGINE_DIR) && \
	cargo run -- validate -l ../$(HERE_DIR)/$(OUT_DIR)/bio.hblib.toml

$(OUT_DIR)/launch-notebook.sh: $(GLUE_DIR)/launch-notebook.sh
	cp $(GLUE_DIR)/launch-notebook.sh $(OUT_DIR)/launch-notebook.sh
