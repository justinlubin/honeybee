use crate::derivation::*;
use crate::ir::*;

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
                    label,
                    antecedents,
                    consequent,
                    ..
                } => computations.push((
                    path.join("_"),
                    consequent,
                    label,
                    antecedents,
                )),
                Tree::Goal(..) => {
                    panic!("invariant violated: non-complete tree")
                }
            }
        }

        write!(f, "# %% Load data\n\n")?;

        for (name, axiom) in initializations {
            write!(f, "{} = ... # {:?}\n", name, axiom)?;
        }

        write!(f, "\n# %% Compute\n\n")?;

        Ok(())
    }
}
