//! # Cell generation
//!
//! This module translates expressions into cells akin to Jupyter notebook
//! cells. In addition to standard code cells, there are hole cells and "choice"
//! cells. A choice cell fills a hole cell with the possible options returned
//! by Programming by Navigation and can be used by the frontend to display
//! the possible steps to take.

use crate::core::*;
use crate::top_down;

use convert_case::Casing;
use indexmap::{IndexMap, IndexSet};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////
// Core types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataChoice {
    pub metadata: IndexMap<String, Value>,
    pub choice_index: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionChoice {
    pub function_title: String,
    pub function_description: Option<String>,
    pub code: Option<String>,
    pub metadata_choices: Vec<MetadataChoice>,
    pub info: Option<toml::Table>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Cell {
    Code {
        title: String,
        description: String,
        code: String,
        open_when_editing: bool,
        open_when_exporting: bool,
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
// Helpers

/// Translate a value to a Python string
pub fn python_value(v: &Value) -> String {
    match v {
        Value::Bool(true) => "True".to_owned(),
        Value::Bool(false) => "False".to_owned(),
        Value::Int(i) => i.to_string(),
        Value::Str(s) => format!("\"{}\"", s).to_owned(),
    }
}

struct Bashify;

impl regex::Replacer for Bashify {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        let overall = &caps[0];
        let indent = overall.len() - overall.trim_start().len();
        let base = format!("!{}", &caps[1].replace(r#"\\"#, r#"\"#).trim());
        let mut lines = vec![];
        let mut current_indent = indent;
        for line in base.split("\n") {
            lines.push(" ".repeat(current_indent) + line.trim());
            current_indent = indent + 4;
        }
        dst.push_str(&lines.join("\n"))
    }
}

fn bashify(s: &str) -> String {
    let re = Regex::new(r#" *__hb_bash\(f"""((.|\n)*?)"""\)"#).unwrap();
    return re.replace_all(&s, Bashify).into();
}

fn make_var_name(s: &str) -> String {
    return s.to_case(convert_case::Case::Constant);
}

fn make_code_preview(code: &str) -> String {
    return bashify(code);
}

////////////////////////////////////////////////////////////////////////////////
// Cell generation

struct Context<'a> {
    library: &'a Library,
    cells: Vec<Cell>,
    fresh_counter: HashMap<String, usize>,
    used_types: IndexSet<MetName>,
    used_functions: IndexSet<BaseFunction>,
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

    fn description(f_sig: &FunctionSignature) -> String {
        let mut ret = "".to_owned();

        match f_sig.info_string("description") {
            Some(desc) => ret += &format!("{}\n\n", desc),
            None => (),
        }

        let mut citations = vec![];

        match f_sig.info_string("citation") {
            Some(cit) => citations.push(cit),
            None => (),
        }

        match f_sig.info_array("additional_citations") {
            Some(cits) => {
                for cit in cits {
                    match cit.as_str() {
                        Some(cit_str) => citations.push(cit_str.to_owned()),
                        None => (),
                    }
                }
            }
            None => (),
        }

        if !citations.is_empty() {
            let plural_suffix = if citations.len() == 1 { "" } else { "s" };
            ret += &format!("### Citation{}\n\n", plural_suffix);
            for cit in citations {
                ret += &format!("- {}\n", cit);
            }
        }

        ret.trim().to_owned()
    }

    fn body_code(
        var_name: &str,
        type_name: &str,
        function_name: &str,
        metadata: &Vec<(String, String)>,
        args: &Vec<(String, String)>,
        implementation: Option<String>,
    ) -> String {
        let mut s = format!("{} = {}(", var_name, type_name);
        let mut needs_newline = false;
        if implementation.is_some() {
            needs_newline = true;
            s += &format!("\n    path=Dir.make(\"{}\"),", function_name);
        }
        if !metadata.is_empty() {
            needs_newline = true;
            s += &metadata
                .into_iter()
                .map(|(lhs, rhs)| format!("\n    {}={},", lhs, rhs))
                .collect::<Vec<_>>()
                .join("");
        }
        if needs_newline {
            s += "\n";
        }
        s += ")";

        match implementation {
            Some(imp) => {
                let mut new_imp = imp.replace("__hb_ret", var_name);

                for (lhs, rhs) in args {
                    new_imp = new_imp.replace(&format!("__hb_{}", lhs), rhs)
                }

                let new_imp = bashify(&new_imp);

                s += &format!("\n\n{}", new_imp)
            }
            None => (),
        };

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
                self.used_functions.insert(f.name.clone());

                let mut arg_strings = vec![];
                for (fp, arg) in args {
                    let mn = f_sig.params.get(fp).unwrap().clone();
                    let arg_sig = self.library.types.get(&mn).unwrap();
                    let arg_var = self.fresh_var(
                        &arg_sig
                            .info_string("var_name")
                            .unwrap_or(make_var_name(&mn.0)),
                    );
                    self.exp(&arg_var, arg);
                    arg_strings.push((fp.0.clone(), arg_var));
                }

                self.cells.push(Cell::Code {
                    title: f_sig
                        .info_string("tite")
                        .unwrap_or(f.name.0.clone()),
                    description: Self::description(f_sig),
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
                    open_when_editing: true,
                    open_when_exporting: true,
                });
            }
        }
    }

    fn preamble(&mut self) {
        // Hyperparameters

        let mut hyperparameters = IndexMap::new();
        for f in self.used_functions.iter().rev() {
            match self
                .library
                .functions
                .get(f)
                .unwrap()
                .info_array("hyperparameters")
            {
                Some(hs) => {
                    for value in hs {
                        let map = value.as_table().unwrap();
                        let name = map
                            .get("name")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned();
                        let default = map
                            .get("default")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned();
                        let comment = map
                            .get("comment")
                            .unwrap()
                            .as_str()
                            .unwrap()
                            .to_owned();
                        hyperparameters.insert(name, (default, comment));
                    }
                }
                None => (),
            }
        }

        let mut hp_code = "".to_owned();

        for (name, (default, comment)) in hyperparameters {
            hp_code += &format!(
                "# PARAMETER: {} (default: {})\n{} = {}\n\n",
                comment, default, name, default
            );
        }

        self.cells.insert(
            0,
            Cell::Code {
                title: "Parameters".to_owned(),
                code: hp_code.trim().to_owned(),
                description: "Before running your code, please set the following parameters!".to_owned(),
                open_when_editing: false,
                open_when_exporting: true,
            },
        );

        // Preamble

        let mut pr_code = "".to_owned();

        match &self.library.preamble {
            Some(pre) => {
                for p in pre {
                    let content = match p.get("content") {
                        Some(c) => c,
                        None => continue,
                    };
                    pr_code += &format!("{}\n\n", content)
                }
            }
            None => (),
        }

        for t in self.used_types.iter().rev() {
            match self.library.types.get(t).unwrap().info_string("code") {
                Some(type_code) => pr_code += &format!("{}\n\n", type_code),
                None => (),
            }
        }

        self.cells.insert(
            1,
            Cell::Code {
                title: "Initialization code".to_owned(),
                code: pr_code.trim().to_owned(),
                description: "".to_owned(),
                open_when_editing: false,
                open_when_exporting: false,
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
        used_functions: IndexSet::new(),
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
                let ret_sig = lib.types.get(&f_sig.ret).unwrap();

                let type_title =
                    ret_sig.info_string("title").unwrap_or(f_sig.ret.0.clone());
                let type_description = ret_sig.info_string("description");

                let function_title =
                    f_sig.info_string("title").unwrap_or(f.name.0.clone());
                let function_description = f_sig.info_string("description");

                let (_, _, fc_map) = ret.entry(*h).or_insert_with(|| {
                    (type_title, type_description, HashMap::new())
                });

                let fc = fc_map.entry(f.name.0.clone()).or_insert_with(|| {
                    FunctionChoice {
                        function_title,
                        function_description,
                        code: f_sig
                            .info_string("code")
                            .map(|s| make_code_preview(&s)),
                        metadata_choices: vec![],
                        info: f_sig.info.clone(),
                    }
                });

                fc.metadata_choices.push(MetadataChoice {
                    metadata: f
                        .metadata
                        .iter()
                        .map(|(mp, v)| (mp.0.clone(), v.clone()))
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
            (
                h,
                (t, d, {
                    let mut v = fmap.into_values().collect::<Vec<_>>();
                    v.sort_by(|fc1, fc2| {
                        fc1.function_title
                            .to_lowercase()
                            .cmp(&fc2.function_title.to_lowercase())
                    });
                    v
                }),
            )
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
