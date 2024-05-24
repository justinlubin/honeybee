use crate::derivation::*;
use crate::ir::*;

use crate::syntax;

pub struct Cells {
    initializations: Vec<(String, String)>,
    computations: Vec<(String, String)>,
}

pub struct Python<'a> {
    tree: &'a Tree,
}

impl<'a> Python<'a> {
    pub fn new(tree: &'a Tree) -> Python<'a> {
        if !tree.complete() {
            panic!("Cannot compile an incomplete tree")
        }
        Python { tree }
    }
}

impl<'a> Python<'a> {
    pub fn emit(&self) -> Cells {
        use std::collections::HashSet;

        let mut initializations = vec![];
        let mut computations = vec![];

        let mut seen_axioms: HashSet<&Fact> = HashSet::new();

        for (path, subtree) in self.tree.postorder() {
            match subtree {
                Tree::Axiom(f) => {
                    if seen_axioms.insert(f) {
                        let name = path.join("_");
                        initializations.push((name, f));
                    }
                }
                Tree::Step {
                    label,
                    antecedents,
                    consequent,
                    ..
                } => {
                    if !path.is_empty() {
                        computations.push((
                            path.join("_"),
                            label,
                            antecedents,
                            consequent,
                        ))
                    }
                }
                Tree::Goal(..) => {
                    panic!("invariant violated: non-complete tree")
                }
            }
        }

        let mut cells = Cells {
            initializations: vec![],
            computations: vec![],
        };

        for (name, axiom) in initializations {
            let args = axiom
                .args
                .iter()
                .map(|(tag, val)| {
                    format!("{}={}", tag, syntax::unparse::value(val))
                })
                .collect::<Vec<_>>()
                .join(",\n    ");

            cells.initializations.push((
                format!("{} = {}(\n    {}\n)", name, axiom.name, args),
                name,
            ));
        }

        for (name, label, antecedents, consequent) in computations {
            let meta_args = consequent
                .args
                .iter()
                .map(|(tag, val)| {
                    format!("{}={}", tag, syntax::unparse::value(val))
                })
                .collect::<Vec<_>>()
                .join(", ");

            let args = antecedents
                .iter()
                .map(|(tag, _)| format!("{}={}_{}", tag, name, tag))
                .collect::<Vec<_>>()
                .join(",\n        ");

            cells.computations.push((
                format!(
                    "{} = {}(\n    m={}.M({}),\n    d={}(\n        {}\n    )\n)",
                    name,
                    consequent.name,
                    consequent.name,
                    meta_args,
                    label,
                    args,
                ),
                name,
            ));
        }

        cells
    }
}

fn nbformat_cell(cell_type: &str, source: &str) -> String {
    format!(
        "{{ \"cell_type\": \"{}\", \"source\": \"{}\" }}",
        cell_type,
        source.replace("\"", "\\\"").replace("\n", "\\n")
    )
}

impl Cells {
    pub fn plain_text(self, lib_imp: &str) -> String {
        format!(
            "{}\n# %% Load data\n\n{}\n\n# %% Compute\n\n{}",
            lib_imp,
            self.initializations
                .into_iter()
                .map(|(src, _name)| src)
                .collect::<Vec<_>>()
                .join("\n"),
            self.computations
                .into_iter()
                .map(|(src, _name)| src)
                .collect::<Vec<_>>()
                .join("\n")
        )
    }

    pub fn nbformat(self, lib_imp: &str) -> String {
        use ipynb::*;
        use std::collections::HashMap;

        let mut cells = vec![];

        cells.push(Cell::Code(CodeCell {
            metadata: HashMap::new(),
            source: vec![lib_imp.trim().to_owned()],
            id: None,
            execution_count: None,
            outputs: vec![],
        }));

        cells.push(Cell::Markdown(MarkdownCell {
            metadata: HashMap::new(),
            id: None,
            attachments: None,
            source: vec!["# Load data".to_owned()],
        }));

        for (src, name) in self.initializations {
            cells.push(Cell::Code(CodeCell {
                metadata: HashMap::new(),
                source: vec![src, name],
                id: None,
                execution_count: None,
                outputs: vec![],
            }))
        }

        cells.push(Cell::Markdown(MarkdownCell {
            metadata: HashMap::new(),
            id: None,
            attachments: None,
            source: vec!["# Run analysis".to_owned()],
        }));

        for (src, name) in self.computations {
            cells.push(Cell::Code(CodeCell {
                metadata: HashMap::new(),
                source: vec![src, name],
                id: None,
                execution_count: None,
                outputs: vec![],
            }))
        }

        serde_json::to_string(&Notebook {
            cells,
            metadata: HashMap::new(),
            nbformat: 4,
            nbformat_minor: 0,
        })
        .unwrap()
    }
}
