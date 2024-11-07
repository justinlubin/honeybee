#![allow(dead_code)]

pub mod analysis;
pub mod benchmark;
pub mod benchmark_data;
pub mod run;

mod backend;
mod derivation;
mod egglog_adapter;
mod enumerate;
mod ir;
mod pbn;
mod syntax;
mod synthesis;
mod task;
mod util;

pub use chumsky::Parser;

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

    match pbn::run(
        &lib,
        &prog,
        analysis::CLI {
            mode: analysis::CLIMode::Auto,
            print_mode: analysis::CLIPrintMode::NoPrint,
        },
    ) {
        Some(tree) => Ok(backend::Python::new(&tree).emit().nbformat(&imp_src)),
        None => Err("Not possible".to_owned()),
    }
}
