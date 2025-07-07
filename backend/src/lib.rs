pub mod main_handler;
pub mod menu;

mod benchmark;
mod cellgen;
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

use codegen::Codegen;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

fn load_problem(
    lib_src: &str,
    prog_src: &str,
) -> Result<core::Problem, String> {
    let library = parse::library(&lib_src)?;
    let program = parse::program(&prog_src)?;
    let problem = core::Problem { library, program };

    typecheck::problem(&problem)
        .map_err(|e| format!("type error: {}", e.message))?;

    Ok(problem)
}

#[wasm_bindgen]
pub fn parse_library(lib_src: &str) -> Result<JsValue, String> {
    let library = parse::library(lib_src)?;
    serde_wasm_bindgen::to_value(&library)
        .map_err(|_| "serde_wasm_bindgen error: to_value(library)".to_owned())
}

#[wasm_bindgen]
pub fn autopilot(lib_src: &str, prog_src: &str) -> Result<String, String> {
    let problem = load_problem(lib_src, prog_src)?;
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

    let gen = codegen::Simple {
        indent: 0,
        color: false,
    };
    gen.exp(&res)
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct ValidGoalMetadataMessage {
    goalName: String,
    choices: Vec<IndexMap<String, core::Value>>,
}

#[wasm_bindgen]
pub fn valid_goal_metadata(
    lib_src: &str,
    prog_src: &str,
) -> Result<JsValue, String> {
    let problem = load_problem(lib_src, prog_src)?;
    let goal_name = problem.program.goal.name.0.clone();

    let engine = egglog::Egglog::new(true);
    let mut oracle = dl_oracle::Oracle::new(engine, problem)?;
    let vgm = oracle.valid_goal_metadata();

    let msg = ValidGoalMetadataMessage {
        goalName: goal_name,
        choices: vgm
            .into_iter()
            .map(|assignment| {
                assignment.into_iter().map(|(k, vs)| (k.0, vs)).collect()
            })
            .collect(),
    };

    serde_wasm_bindgen::to_value(&msg).map_err(|_| {
        "serde_wasm_bindgen error in valid_goal_metadata".to_owned()
    })
}

////////////////////////////////////////////////////////////////////////////////
// PBN Interaction

struct State {
    controller:
        pbn::Controller<top_down::TopDownStep<core::ParameterizedFunction>>,
    library: core::Library,
}

static mut STATE: Option<State> = None;

#[allow(static_mut_refs)]
fn get_state() -> Result<&'static mut State, String> {
    match unsafe { STATE.as_mut() } {
        Some(c) => Ok(c),
        None => Err("must call pbn_init first".to_owned()),
    }
}

fn set_state(state: State) {
    unsafe {
        STATE = Some(state);
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct PbnStatusMessage {
    cells: Vec<cellgen::Cell>,
    output: Option<String>,
}

fn send_message() -> Result<JsValue, String> {
    let state = get_state()?;

    let options = state.controller.provide().map_err(|e| format!("{:?}", e))?;

    let work_exp = state.controller.working_expression();

    let msg = PbnStatusMessage {
        cells: cellgen::fill(
            &state.library,
            &options,
            cellgen::exp(&state.library, &work_exp),
        )?,
        output: if state.controller.valid() {
            Some(codegen::plain_text_notebook(&state.library, &work_exp))
        } else {
            None
        },
    };

    serde_wasm_bindgen::to_value(&msg)
        .map_err(|_| "serde_wasm_bindgen error in send_message".to_owned())
}

#[wasm_bindgen]
pub fn pbn_init(lib_src: &str, prog_src: &str) -> Result<JsValue, String> {
    let problem = load_problem(lib_src, prog_src)?;
    let timer = util::Timer::infinite();
    let algorithm = menu::Algorithm::PBNHoneybee;

    set_state(State {
        library: problem.library.clone(),
        controller: algorithm.controller(timer, problem),
    });

    send_message()
}

#[wasm_bindgen]
pub fn pbn_choose(choice_index: usize) -> Result<JsValue, String> {
    let state = get_state()?;
    let mut options =
        state.controller.provide().map_err(|e| format!("{:?}", e))?;
    state.controller.decide(options.swap_remove(choice_index));
    send_message()
}
