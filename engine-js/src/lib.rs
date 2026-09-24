use honeybee::{cellgen, codegen, core, dl_oracle, egglog, menu, parse, top_down, typecheck};

use pbn;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

fn load_problem(lib_src: &str, prog_src: &str) -> Result<core::Problem, String> {
    let library = parse::library(&lib_src)?;
    let program = parse::program(&prog_src)?;
    let problem = core::Problem { library, program };

    typecheck::problem(&problem).map_err(|e| format!("type error: {}", e.message))?;

    Ok(problem)
}

// Bit of a hack; find some placeholder goal we can slot at the end of the props
fn find_placeholder_goal_name(lib: &core::Library) -> Option<&str> {
    for (name, sig) in &lib.types {
        if sig.params.len() != 0 {
            continue;
        }
        return Some(&name.0);
    }

    None
}

fn load_problem_without_goal(
    lib_src: &str,
    props_src: &str,
) -> Result<(core::Library, Vec<core::Met<core::Value>>), String> {
    let library = parse::library(&lib_src)?;

    let goal_name = find_placeholder_goal_name(&library)
        .ok_or("Cannot find suitable placeholder goal".to_owned())?;

    let prog_src = format!(
        "{}\n\n[Goal]\nname = \"{}\"\nargs = {{}}",
        props_src, goal_name
    );

    let program = parse::program(&prog_src)?;
    let problem = core::Problem { library, program };

    typecheck::problem(&problem).map_err(|e| format!("type error: {}", e.message))?;

    Ok((problem.library, problem.program.props))
}

#[wasm_bindgen]
pub fn parse_library(lib_src: &str) -> Result<JsValue, String> {
    let library = parse::library(lib_src)?;
    serde_wasm_bindgen::to_value(&library)
        .map_err(|_| "serde_wasm_bindgen error: to_value(library)".to_owned())
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
struct ValidGoalMetadataMessage {
    goals: IndexMap<String, Vec<IndexMap<String, core::Value>>>,
}

// This function is quite fragile, be careful!
#[wasm_bindgen]
pub fn valid_goal_metadata(lib_src: &str, props_src: &str) -> Result<JsValue, String> {
    let (library, props) = load_problem_without_goal(lib_src, props_src)?;

    let mut goals = IndexMap::new();

    for (goal_name, _) in &library.types {
        // The goal args here are incorrect, but it's ok just for a metadata check!
        let problem = core::Problem {
            library: library.clone(),
            program: core::Program {
                props: props.clone(),
                goal: core::Met {
                    name: goal_name.clone(),
                    args: IndexMap::new(),
                },
            },
        };

        let engine = egglog::Egglog::new(true);
        let mut oracle = dl_oracle::Oracle::new(engine, problem)?;
        let vgm = oracle.valid_goal_metadata();

        goals.insert(
            goal_name.0.clone(),
            vgm.into_iter()
                .map(|assignment| assignment.into_iter().map(|(k, vs)| (k.0, vs)).collect())
                .collect(),
        );
    }

    let msg = ValidGoalMetadataMessage { goals };

    serde_wasm_bindgen::to_value(&msg)
        .map_err(|_| "serde_wasm_bindgen error in valid_goal_metadata".to_owned())
}

////////////////////////////////////////////////////////////////////////////////
// PBN Interaction

struct State {
    controller:
        pbn::Controller<honeybee::util::Timer, top_down::TopDownStep<core::ParameterizedFunction>>,
    library: core::Library,
    sound: bool,
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
    can_undo: bool,
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
            Some(codegen::jupyter_notebook(&state.library, &work_exp))
        } else {
            None
        },
        can_undo: state.controller.can_undo(),
    };

    serde_wasm_bindgen::to_value(&msg)
        .map_err(|_| "serde_wasm_bindgen error in send_message".to_owned())
}

#[wasm_bindgen]
pub fn pbn_init(lib_src: &str, prog_src: &str, sound: bool) -> Result<JsValue, String> {
    let problem = load_problem(lib_src, prog_src)?;
    let timer = honeybee::util::Timer::infinite();
    let algorithm = if sound {
        menu::Algorithm::PBNHoneybee
    } else {
        menu::Algorithm::Unsound
    };

    set_state(State {
        library: problem.library.clone(),
        controller: algorithm.controller(timer, problem, true),
        sound,
    });

    send_message()
}

#[wasm_bindgen]
pub fn pbn_choose(choice_index: usize) -> Result<JsValue, String> {
    let state = get_state()?;
    let mut options = state.controller.provide().map_err(|e| format!("{:?}", e))?;
    state.controller.decide(options.swap_remove(choice_index));

    if state.sound {
        // Check for auto-decisions (F_* functions)
        'fixpoint: loop {
            let mut options = state.controller.provide().map_err(|e| format!("{:?}", e))?;
            for (i, option) in options.iter().enumerate() {
                match option {
                    top_down::TopDownStep::Extend(_, f, _) => {
                        if f.name.0.starts_with("F_") {
                            state
                                .controller
                                .decide_without_history(options.swap_remove(i));
                            continue 'fixpoint;
                        }
                    }
                    _ => continue,
                }
            }
            break;
        }
    }

    send_message()
}

#[wasm_bindgen]
pub fn pbn_undo() -> Result<JsValue, String> {
    let state = get_state()?;
    if !state.controller.can_undo() {
        return Err("cannot undo".to_owned());
    }
    state.controller.undo();
    send_message()
}
