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
mod machine_readable;
mod parse;
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
    controller: pbn::Controller<
        util::Timer,
        top_down::TopDownStep<core::ParameterizedFunction>,
    >,
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
pub fn pbn_init(lib_src: &str, prog_src: &str) -> Result<JsValue, String> {
    let problem = load_problem(lib_src, prog_src)?;
    let timer = util::Timer::infinite();
    let algorithm = menu::Algorithm::PBNHoneybee;

    set_state(State {
        library: problem.library.clone(),
        controller: algorithm.controller(timer, problem, true),
    });

    send_message()
}

#[wasm_bindgen]
pub fn pbn_choose(choice_index: usize) -> Result<JsValue, String> {
    let state = get_state()?;
    let mut options =
        state.controller.provide().map_err(|e| format!("{:?}", e))?;
    state.controller.decide(options.swap_remove(choice_index));

    // Check for auto-decisions (F_* functions)
    'fixpoint: loop {
        let mut options =
            state.controller.provide().map_err(|e| format!("{:?}", e))?;
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

////////////////////////////////////////////////////////////////////////////////
// Python bindings

#[pyo3::pymodule]
mod honeybee {
    use crate::unparse;

    use super::{cellgen, core, menu, parse, top_down, typecheck, util};

    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use pythonize::pythonize;

    #[pyclass(unsendable)]
    struct Controller {
        _controller: pbn::Controller<
            util::Timer,
            top_down::TopDownStep<core::ParameterizedFunction>,
        >,
        _library: core::Library,
    }

    fn load_problem(
        library: &str,
        program: &str,
    ) -> pyo3::PyResult<core::Problem> {
        let lib_string = std::fs::read_to_string(library).map_err(|e| {
            PyValueError::new_err(format!(
                "error while reading library file: {}",
                e.to_string()
            ))
        })?;

        let prog_string = std::fs::read_to_string(program).map_err(|e| {
            PyValueError::new_err(format!(
                "error while reading program file: {}",
                e.to_string()
            ))
        })?;

        let library = parse::library(&lib_string).map_err(|e| {
            PyValueError::new_err(format!("parse error (library):\n{}", e))
        })?;

        let program = parse::program(&prog_string).map_err(|e| {
            PyValueError::new_err(format!("parse error (program):\n{}", e))
        })?;

        let problem = core::Problem { library, program };

        typecheck::problem(&problem).map_err(|e| {
            PyValueError::new_err(format!(
                "type error: {}\n  occurred:{}",
                e.message,
                e.context
                    .into_iter()
                    .map(|ctx| format!("\n    - in {}", ctx))
                    .collect::<Vec<_>>()
                    .join("")
            ))
        })?;

        Ok(problem)
    }

    fn out_of_time() -> PyErr {
        PyValueError::new_err("Out of time")
    }

    fn no_more_steps() -> PyErr {
        PyValueError::new_err("No more steps")
    }

    #[pymethods]
    impl Controller {
        #[new]
        #[pyo3(signature = (library, program, algorithm = "PBNHoneybee"))]
        fn new(
            library: &str,
            program: &str,
            algorithm: &str,
        ) -> PyResult<Self> {
            let problem = load_problem(library, program)?;
            let timer = util::Timer::infinite();
            let algorithm: menu::Algorithm =
                algorithm.parse().map_err(|_| {
                    PyValueError::new_err(format!(
                        "Unknown algorithm '{}'",
                        algorithm
                    ))
                })?;

            Ok(Self {
                _library: problem.library.clone(),
                _controller: algorithm.controller(timer, problem, false),
            })
        }

        fn working_expression(&self, py: Python) -> Py<PyAny> {
            pythonize(py, self._controller.working_expression())
                .unwrap()
                .unbind()
        }

        fn provide(&mut self, py: Python) -> PyResult<Vec<Py<PyAny>>> {
            let options =
                self._controller.provide().map_err(|_| out_of_time())?;

            let function_choices: Vec<_> = cellgen::fill(
                &self._library,
                &options,
                cellgen::exp(
                    &self._library,
                    &self._controller.working_expression(),
                ),
            )
            .unwrap()
            .into_iter()
            .find_map(|c| match c {
                cellgen::Cell::Choice {
                    function_choices, ..
                } => Some(function_choices),
                _ => None,
            })
            .ok_or_else(|| no_more_steps())?
            .iter()
            .map(|fc| pythonize(py, fc).unwrap().unbind())
            .collect();

            Ok(function_choices)
        }

        fn decide(&mut self, index: usize) -> PyResult<()> {
            let mut options =
                self._controller.provide().map_err(|_| out_of_time())?;
            if index >= options.len() {
                return Err(PyValueError::new_err(format!(
                    "Index out of bounds for options: {} (length = {})",
                    index,
                    options.len()
                )));
            }
            self._controller.decide(options.swap_remove(index));
            Ok(())
        }
    }
}
