mod ir;
mod syntax;

use chumsky::Parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_library(lib: String) -> JsValue {
    let lib = syntax::parse::library().parse(lib).unwrap();
    serde_wasm_bindgen::to_value(&lib).unwrap()
}
