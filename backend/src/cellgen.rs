//! # Cell generation
//!
//! This module translates expressions into cells akin to Jupyter notebook
//! cells. In addition to standard code cells, there are hole cells and "choice"
//! cells. A choice cell fills a hole cell with the possible options returned
//! by Programming by Navigation and can be used by the frontend to display
//! the possible steps to take.

use crate::core::*;
use crate::top_down;

use indexmap::{IndexMap, IndexSet};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Core types

pub struct MetadataChoice {
    pub metadata: IndexMap<String, String>,
    pub choice_index: usize,
}

pub struct FunctionChoice {
    pub function_title: String,
    pub function_description: Option<String>,
    pub code: Option<String>,
    pub metadata_choices: Vec<MetadataChoice>,
}

pub enum Cell {
    Code {
        var_name: Option<String>,
        type_title: Option<String>,
        type_description: Option<String>,
        function_title: Option<String>,
        function_description: Option<String>,
        title: Option<String>,
        code: String,
    },
    Hole {
        var_name: String,
        hole_name: top_down::HoleName,
    },
    Choice {
        var_name: String,
        type_title: String,
        type_description: Option<String>,
        function_choices: Vec<FunctionChoice>,
    },
}

////////////////////////////////////////////////////////////////////////////////
// Basic helpers

/// Translate a value to a Python string
pub fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

////////////////////////////////////////////////////////////////////////////////
// Cell generation

struct Context<'a> {
    library: &'a Library,
    cells: Vec<Cell>,
    fresh_counter: HashMap<String, usize>,
    used_types: IndexSet<MetName>,
}

impl<'a> Context<'a> {
    fn fresh_var(&mut self, prefix: &str) -> String {
        let c = self.fresh_counter.entry(prefix.to_owned()).or_insert(1);
        let s = format!(
            "{}{}",
            prefix,
            if *c > 1 {
                format!("{}", *c)
            } else {
                "".to_owned()
            }
        );
        *c += 1;
        s
    }

    fn body_code(
        var_name: &str,
        type_name: &str,
        function_name: &str,
        metadata: &Vec<(String, String)>,
        args: &Vec<(String, String)>,
        implementation: Option<String>,
    ) -> String {
        let mut s = "".to_owned();

        match implementation {
            Some(imp) => s += &format!("{}\n\n", imp),
            None => (),
        };

        let mut static_val = format!("{}.S(", type_name);
        static_val += &metadata
            .into_iter()
            .map(|(lhs, rhs)| format!("{}={}", lhs, rhs))
            .collect::<Vec<_>>()
            .join(", ");
        static_val += ")";

        s += &format!("{} = {}(\n    static=", var_name, type_name);
        s += &static_val;
        s += &format!(",\n    dynamic={}(", function_name);
        s += &args
            .into_iter()
            .map(|(lhs, rhs)| format!("{}={}, ", lhs, rhs))
            .collect::<Vec<_>>()
            .join("");
        s += &format!("ret={}", static_val);
        s += &format!("),\n)\n\n{}", var_name);

        s
    }

    fn exp(&mut self, var_name: &str, e: &Exp) {
        match e {
            top_down::Sketch::Hole(h) => {
                self.cells.push(Cell::Hole {
                    var_name: var_name.to_owned(),
                    hole_name: *h,
                });
            }
            top_down::Sketch::App(f, args) => {
                let f_sig = self.library.functions.get(&f.name).unwrap();
                self.used_types.insert(f_sig.ret.clone());

                let ret_sig = self.library.types.get(&f_sig.ret).unwrap();

                let mut arg_strings = vec![];
                for (fp, arg) in args {
                    let mn = f_sig.params.get(fp).unwrap().clone();
                    let arg_sig = self.library.types.get(&mn).unwrap();
                    let arg_var = self.fresh_var(
                        &arg_sig
                            .info_string("var_name")
                            .unwrap_or(mn.0.to_uppercase().to_owned()),
                    );
                    self.exp(&arg_var, arg);
                    arg_strings.push((fp.0.clone(), arg_var));
                }

                self.cells.push(Cell::Code {
                    var_name: Some(var_name.to_owned()),
                    type_title: Some(
                        ret_sig
                            .info_string("title")
                            .unwrap_or(f_sig.ret.0.clone()),
                    ),
                    type_description: ret_sig.info_string("description"),
                    function_title: Some(
                        f_sig.info_string("title").unwrap_or(f.name.0.clone()),
                    ),
                    function_description: f_sig.info_string("description"),
                    title: None,
                    code: Self::body_code(
                        var_name,
                        &f_sig.ret.0,
                        &f.name.0,
                        &f.metadata
                            .iter()
                            .map(|(mp, v)| (mp.0.clone(), python_value(v)))
                            .collect(),
                        &arg_strings,
                        f_sig.info_string("code"),
                    ),
                });
            }
        }
    }

    fn preamble(&mut self) {
        let mut code = "".to_owned();

        match &self.library.preamble {
            Some(pre) => {
                for p in pre {
                    let content = match p.get("content") {
                        Some(c) => c,
                        None => continue,
                    };
                    code += &format!("{}\n\n", content)
                }
            }
            None => (),
        }

        for t in self.used_types.iter().rev() {
            match self.library.types.get(t).unwrap().info_string("code") {
                Some(type_code) => code += &format!("{}\n\n", type_code),
                None => (),
            }
        }

        self.cells.insert(
            0,
            Cell::Code {
                var_name: None,
                type_title: None,
                type_description: None,
                function_title: None,
                function_description: None,
                title: Some("Helpers and types".to_owned()),
                code: code.trim().to_owned(),
            },
        );
    }
}

pub fn exp(library: &Library, e: &Exp) -> Vec<Cell> {
    let mut ctx = Context {
        library,
        cells: vec![],
        fresh_counter: HashMap::new(),
        used_types: IndexSet::new(),
    };

    ctx.exp("GOAL", e);
    ctx.preamble();

    ctx.cells
}

////////////////////////////////////////////////////////////////////////////////
// Cell filling

fn collate_choices(
    lib: &Library,
    choices: &Vec<top_down::TopDownStep<ParameterizedFunction>>,
) -> Result<
    HashMap<top_down::HoleName, (String, Option<String>, Vec<FunctionChoice>)>,
    String,
> {
    let mut ret: HashMap<
        top_down::HoleName,
        (String, Option<String>, HashMap<String, FunctionChoice>),
    > = HashMap::new();

    for (choice_index, choice) in choices.iter().enumerate() {
        match choice {
            top_down::TopDownStep::Extend(h, f, _args) => {
                let f_sig = lib.functions.get(&f.name).unwrap();

                let function_title =
                    f_sig.info_string("title").unwrap_or(f.name.0.clone());
                let function_description = f_sig.info_string("description");

                let (_, _, fc_map) = ret.entry(*h).or_insert_with(|| {
                    (
                        function_title.clone(),
                        function_description.clone(),
                        HashMap::new(),
                    )
                });

                let fc = fc_map.entry(f.name.0.clone()).or_insert_with(|| {
                    FunctionChoice {
                        function_title,
                        function_description,
                        code: f_sig.info_string("code"),
                        metadata_choices: vec![],
                    }
                });

                fc.metadata_choices.push(MetadataChoice {
                    metadata: f
                        .metadata
                        .iter()
                        .map(|(mp, v)| (mp.0.clone(), python_value(v)))
                        .collect(),
                    choice_index,
                });
            }
            top_down::TopDownStep::Seq(..) => {
                return Err("Sequenced steps unsupported".to_owned())
            }
        }
    }

    Ok(ret
        .into_iter()
        .map(|(h, (t, d, fmap))| {
            (h, (t, d, fmap.into_values().collect::<Vec<_>>()))
        })
        .collect::<HashMap<_, _>>())
}

pub fn fill(
    lib: &Library,
    choices: &Vec<top_down::TopDownStep<ParameterizedFunction>>,
    mut cells: Vec<Cell>,
) -> Result<Vec<Cell>, String> {
    let mut collated_choices = collate_choices(lib, choices)?;
    for cell in &mut cells {
        match cell {
            Cell::Hole {
                var_name,
                hole_name,
            } => {
                let (type_title, type_description, function_choices) =
                    collated_choices.remove(hole_name).ok_or(
                        format!("No choices for hole {}", hole_name).to_owned(),
                    )?;

                *cell = Cell::Choice {
                    var_name: std::mem::take(var_name),
                    type_title,
                    type_description,
                    function_choices,
                }
            }
            Cell::Choice { .. } => {
                return Err(
                    "Cannot fill cells that already have choices".to_owned()
                )
            }
            Cell::Code { .. } => (),
        }
    }
    Ok(cells)
}
