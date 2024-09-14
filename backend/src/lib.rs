mod analysis;
mod backend;
mod derivation;
mod egglog_adapter;
mod ir;
mod syntax;
mod synthesis;

mod top_level;

use chumsky::Parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_library(lib: String) -> JsValue {
    let lib = syntax::parse::library().parse(lib).unwrap();
    serde_wasm_bindgen::to_value(&lib).unwrap()
}

#[wasm_bindgen]
pub fn generate_notebook(
    lib_src: String,
    imp_src: String,
    prog_src: String,
) -> Result<String, String> {
    let lib = syntax::parse::library()
        .parse(lib_src)
        .map_err(|_| "Library parse error")?;
    let prog = syntax::parse::program()
        .parse(prog_src)
        .map_err(|_| "Program parse error")?;

    let runner = top_level::Runner { interactive: false };

    match runner.run(lib, &imp_src, prog) {
        Some(output) => Ok(output),
        None => Err("Not possible".to_owned()),
    }
}
