use crate::derivation::*;
use crate::ir::*;

use crate::syntax;

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

impl<'a> std::fmt::Display for Python<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
                    label, antecedents, ..
                } => {
                    if !path.is_empty() {
                        computations.push((path.join("_"), label, antecedents))
                    }
                }
                Tree::Goal(..) => {
                    panic!("invariant violated: non-complete tree")
                }
            }
        }

        write!(f, "# %% Load data\n\n")?;

        for (name, axiom) in initializations {
            let args = axiom
                .args
                .iter()
                .map(|(tag, val)| {
                    format!("{}={}", tag, syntax::unparse::value(val))
                })
                .collect::<Vec<_>>()
                .join(", ");

            write!(f, "{} = {}({})\n", name, axiom.name, args)?;
        }

        write!(f, "\n# %% Compute\n")?;

        for (name, label, antecedents) in computations {
            let args = antecedents
                .iter()
                .map(|(tag, _)| format!("{}={}_{}", tag, name, tag))
                .collect::<Vec<_>>()
                .join(", ");

            write!(f, "\n{} = {}({})", name, label, args)?;
        }

        Ok(())
    }
}
