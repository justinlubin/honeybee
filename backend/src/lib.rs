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

    Ok(codegen::simple_multi(&res, 0, false))
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

static mut STATE: Option<
    pbn::Controller<top_down::TopDownStep<core::ParameterizedFunction>>,
> = None;

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct PbnStatusMessage {
    workingExpression: String,
    choices: Vec<(usize, String)>,
    valid: bool,
}

#[allow(static_mut_refs)]
fn get_controller() -> Result<
    &'static mut pbn::Controller<
        top_down::TopDownStep<core::ParameterizedFunction>,
    >,
    String,
> {
    match unsafe { STATE.as_mut() } {
        Some(c) => Ok(c),
        None => Err("must call pbn_init first".to_owned()),
    }
}

fn send_message() -> Result<JsValue, String> {
    let controller = get_controller()?;

    let options = controller.provide().map_err(|e| format!("{:?}", e))?;

    // TODO: Need to handle metadata values
    let mut choices = vec![];

    for opt in options {
        match opt {
            top_down::TopDownStep::Extend(h, f, _) => {
                choices.push((h, f.name.0))
            }
            top_down::TopDownStep::Seq(..) => {
                return Err("sequenced steps unsupported".to_owned())
            }
        }
    }

    let msg = PbnStatusMessage {
        workingExpression: codegen::simple_multi(
            &controller.working_expression(),
            0,
            false,
        ),
        choices,
        valid: controller.valid(),
    };

    serde_wasm_bindgen::to_value(&msg)
        .map_err(|_| "serde_wasm_bindgen error in send_message".to_owned())
}

#[wasm_bindgen]
pub fn pbn_init(lib_src: &str, prog_src: &str) -> Result<JsValue, String> {
    let problem = load_problem(lib_src, prog_src)?;
    let timer = util::Timer::infinite();
    let algorithm = menu::Algorithm::PBNHoneybee;

    unsafe {
        STATE = Some(algorithm.controller(timer, problem));
    }

    send_message()
}

#[wasm_bindgen]
pub fn pbn_choose(choice: usize) -> Result<JsValue, String> {
    let controller = get_controller()?;
    let mut options = controller.provide().map_err(|e| format!("{:?}", e))?;
    controller.decide(options.swap_remove(choice));
    send_message()
}
