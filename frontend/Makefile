OPTIMIZE_ELM := false
OUT_DIR := www
CURRENT_COMMIT := $(shell git rev-parse --short HEAD)

DIAGRAMS_DIR := $(OUT_DIR)/assets/diagrams
DIAGRAM_OBJECTS := $(patsubst \
    $(DIAGRAMS_DIR)/dot/%.dot, \
    $(DIAGRAMS_DIR)/svg/%.svg, \
    $(wildcard $(DIAGRAMS_DIR)/dot/*.dot))

.PHONY: all
all: \
	$(OUT_DIR)/elm.js \
	$(OUT_DIR)/pkg/honeybee.js \
	$(OUT_DIR)/std-bio.hblib.toml \
	$(DIAGRAM_OBJECTS)

.PHONY: clean
clean:
	rm $(OUT_DIR)/elm.js
	rm $(OUT_DIR)/pkg/honeybee.js
	rm $(OUT_DIR)/std-bio.hblib.toml
	rm $(OUT_DIR)/assets/diagrams/svg/*.svg

$(OUT_DIR)/elm.js: $(wildcard src/*.elm)
ifeq ($(OPTIMIZE_ELM),true)
	elm make src/Main.elm --optimize --output=$(OUT_DIR)/elm.js || touch $(OUT_DIR)/elm.js
	uglifyjs $(OUT_DIR)/elm.js --compress "pure_funcs=[F2,F3,F4,F5,F6,F7,F8,F9,A2,A3,A4,A5,A6,A7,A8,A9],pure_getters,keep_fargs=false,unsafe_comps,unsafe" | uglifyjs --mangle --output $(OUT_DIR)/elm.min.js
	mv $(OUT_DIR)/elm.min.js $(OUT_DIR)/elm.js
else
	elm make src/Main.elm --output=$(OUT_DIR)/elm.js || touch $(OUT_DIR)/elm.js
endif
	sed "s/<<<COMMIT-SHORT-HASH>>>/$(CURRENT_COMMIT)/g" $(OUT_DIR)/elm.js > $(OUT_DIR)/elm.js.tmp
	mv $(OUT_DIR)/elm.js.tmp $(OUT_DIR)/elm.js

$(OUT_DIR)/pkg/honeybee.js: $(wildcard ../backend/src/*.rs)
	cd ../backend/ && \
	cargo build && \
	wasm-pack build --target web --out-dir ../frontend/www/pkg

$(OUT_DIR)/std-bio.hblib.toml: $(wildcard ../library-generator/*.py)
	uv run \
		--project ../library-generator \
		ty check && \
	uv run \
		--project ../library-generator/ \
		../library-generator/std-bio.py \
		> $(OUT_DIR)/std-bio.hblib.toml && \
	cd ../backend/ && \
	cargo run -- validate -l ../frontend/$(OUT_DIR)/std-bio.hblib.toml

$(OUT_DIR)/assets/diagrams/svg/%.svg: $(OUT_DIR)/assets/diagrams/dot/%.dot

$(DIAGRAMS_DIR)/svg/%.svg: $(DIAGRAMS_DIR)/dot/%.dot
	dot -Tsvg $< > $@
