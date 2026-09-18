# Requirements:
#   cargo
#   elm
#   uglifyjs
#   uv
#   wasm-bindgen
#   wasm32-unknown-unknown Rust target
#     rustup target add wasm32-unknown-unknown

OPTIMIZE_ELM := false
HERE_DIR := notebook-editor
ENGINE_DIR := ../engine
CLI_DIR := ../engine-cli
JS_DIR := ../engine-js
LANG_DIR := ../honey
GLUE_DIR := ../libs/bio
OUT_DIR := www
CURRENT_COMMIT := $(shell git rev-parse --short HEAD)

.PHONY: all
all: \
	$(OUT_DIR)/elm.js \
	$(OUT_DIR)/pkg \
	$(OUT_DIR)/bio.hblib.toml \
	$(OUT_DIR)/launch-notebook.sh

.PHONY: clean
clean:
	rm -f $(OUT_DIR)/elm.js
	rm -rf $(OUT_DIR)/pkg
	rm -f $(OUT_DIR)/bio.hblib.toml
	rm -f $(OUT_DIR)/launch-notebook.sh

# Depends on honeybee.js to ensure versions are in-sync
$(OUT_DIR)/elm.js: $(wildcard src/*.elm) $(OUT_DIR)/pkg
ifeq ($(OPTIMIZE_ELM),true)
	elm make src/Main.elm --optimize --output=$(OUT_DIR)/elm.js || touch $(OUT_DIR)/elm.js
	uglifyjs $(OUT_DIR)/elm.js --compress "pure_funcs=[F2,F3,F4,F5,F6,F7,F8,F9,A2,A3,A4,A5,A6,A7,A8,A9],pure_getters,keep_fargs=false,unsafe_comps,unsafe" | uglifyjs --mangle --output $(OUT_DIR)/elm.min.js
	mv $(OUT_DIR)/elm.min.js $(OUT_DIR)/elm.js
else
	elm make src/Main.elm --output=$(OUT_DIR)/elm.js || touch $(OUT_DIR)/elm.js
endif
	sed "s/<<<COMMIT-SHORT-HASH>>>/$(CURRENT_COMMIT)/g" $(OUT_DIR)/elm.js \
	| if [ -n "${PUBLISH_STABLE}" ]; then sed "s/<<<UNSTABLE-INDICATOR>>>/STABLE/g"; else sed "s/<<<UNSTABLE-INDICATOR>>>/UNSTABLE/g"; fi \
	> $(OUT_DIR)/elm.js.tmp
	mv $(OUT_DIR)/elm.js.tmp $(OUT_DIR)/elm.js

$(OUT_DIR)/pkg: $(wildcard $(JS_DIR)/src/*.rs)
	cd $(JS_DIR) && \
	make pkg && \
	rm -rf $(OUT_DIR)/pkg && \
	cp -r pkg ../$(HERE_DIR)/$(OUT_DIR)/pkg

$(OUT_DIR)/bio.hblib.toml: $(wildcard $(LANG_DIR)/*.py) $(wildcard $(GLUE_DIR)/*.py)
	cd $(GLUE_DIR) && ty check
	uv run \
		--project $(GLUE_DIR) \
		$(GLUE_DIR)/bio.py \
		> $(OUT_DIR)/bio.hblib.toml && \
	cd $(CLI_DIR) && \
	cargo run -- validate -l ../$(HERE_DIR)/$(OUT_DIR)/bio.hblib.toml

$(OUT_DIR)/launch-notebook.sh: $(GLUE_DIR)/launch-notebook.sh
	cp $(GLUE_DIR)/launch-notebook.sh $(OUT_DIR)/launch-notebook.sh
