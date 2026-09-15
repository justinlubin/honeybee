#[pyo3::pymodule]
mod honeybee {
    use crate::unparse;

    use super::{cellgen, core, menu, parse, top_down, typecheck, util};

    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use pythonize::pythonize;

    #[pyclass(unsendable)]
    struct Controller {
        _controller:
            pbn::Controller<util::Timer, top_down::TopDownStep<core::ParameterizedFunction>>,
        _library: core::Library,
    }

    fn load_problem(library: &str, program: &str) -> pyo3::PyResult<core::Problem> {
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

        let library = parse::library(&lib_string)
            .map_err(|e| PyValueError::new_err(format!("parse error (library):\n{}", e)))?;

        let program = parse::program(&prog_string)
            .map_err(|e| PyValueError::new_err(format!("parse error (program):\n{}", e)))?;

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

    #[pymethods]
    impl Controller {
        #[new]
        #[pyo3(signature = (library, program, algorithm = "PBNHoneybee"))]
        fn new(library: &str, program: &str, algorithm: &str) -> PyResult<Self> {
            let problem = load_problem(library, program)?;
            let timer = util::Timer::infinite();
            let algorithm: menu::Algorithm = algorithm
                .parse()
                .map_err(|_| PyValueError::new_err(format!("Unknown algorithm '{}'", algorithm)))?;

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
            let options = self._controller.provide().map_err(|_| out_of_time())?;

            let function_choices: Vec<_> = cellgen::fill(
                &self._library,
                &options,
                cellgen::exp(&self._library, &self._controller.working_expression()),
            )
            .unwrap()
            .into_iter()
            .find_map(|c| match c {
                cellgen::Cell::Choice {
                    function_choices, ..
                } => Some(function_choices),
                _ => None,
            })
            .unwrap_or_else(|| vec![])
            .iter()
            .map(|fc| pythonize(py, fc).unwrap().unbind())
            .collect();

            Ok(function_choices)
        }

        fn decide(&mut self, index: usize) -> PyResult<()> {
            let mut options = self._controller.provide().map_err(|_| out_of_time())?;
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
