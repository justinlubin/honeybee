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
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
