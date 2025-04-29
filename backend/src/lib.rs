pub mod main_handler;
pub mod menu;

mod benchmark;
mod codegen;
mod core;
mod datalog;
mod dl_oracle;
mod egglog;
mod enumerate;
mod eval;
mod parse;
mod pbn;
mod top_down;
mod traditional_synthesis;
mod typecheck;
mod unparse;
mod util;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse_library(lib_src: &str) -> JsValue {
    serde_wasm_bindgen::to_value(&parse::library(lib_src)).unwrap()
}

#[wasm_bindgen]
pub fn autopilot(lib_src: &str, prog_src: &str) -> Result<String, String> {
    let library = parse::library(&lib_src)?;
    let program = parse::program(&prog_src)?;
    let problem = core::Problem { library, program };

    let timer = util::Timer::infinite();
    let start = top_down::Sketch::blank();
    let mut synth = menu::Algorithm::PBNHoneybee.any_synthesizer(problem);
    let hf = match synth.provide_any(&timer, &start) {
        Ok(Some(hf)) => hf,
        Ok(None) => return Err("no solution".to_owned()),
        Err(e) => return Err(format!("{:?}", e)),
    };

    let mut res = start;

    for (lhs, rhs) in hf {
        res = res.substitute(lhs, &rhs);
    }

    Ok(codegen::python_multi(&res, 0))
}
