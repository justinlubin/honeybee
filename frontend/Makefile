OPTIMIZE_ELM = false
OUT_DIR = www

$(OUT_DIR)/elm.js: $(wildcard src/*.elm)
ifeq ($(OPTIMIZE_ELM),true)
	elm make src/Main.elm --optimize --output=$(OUT_DIR)/elm.js
	uglifyjs $(OUT_DIR)/elm.js --compress "pure_funcs=[F2,F3,F4,F5,F6,F7,F8,F9,A2,A3,A4,A5,A6,A7,A8,A9],pure_getters,keep_fargs=false,unsafe_comps,unsafe" | uglifyjs --mangle --output $(OUT_DIR)/elm.min.js
	mv $(OUT_DIR)/elm.min.js $(OUT_DIR)/elm.js
else
	elm make src/Main.elm --output=$(OUT_DIR)/elm.js
endif
