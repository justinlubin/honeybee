use crate::ir::*;

pub type Path = Vec<String>;

pub enum Tree {
    Axiom(Fact),
    Goal(Fact),
    Step {
        label: String,
        antecedents: Vec<(String, Tree)>,
        consequent: Fact,
    },
}

impl Tree {
    fn replace(&mut self, mut path: Path, subtree: Tree) {
        match path.pop() {
            Some(name) => match self {
                Tree::Step { antecedents, .. } => antecedents
                    .iter_mut()
                    .find_map(|(n, t)| if name == *n { Some(t) } else { None })
                    .unwrap()
                    .replace(path, subtree),
                _ => panic!(),
            },
            None => *self = subtree,
        }
    }

    fn goals(&self) -> Vec<(Path, &Fact)> {
        match self {
            Tree::Axiom(_) => vec![],
            Tree::Goal(fact) => vec![(vec![], fact)],
            Tree::Step {
                antecedents, label, ..
            } => antecedents
                .iter()
                .flat_map(|(_, subtree)| {
                    subtree.goals().into_iter().map(|(mut path, fact)| {
                        path.push(label.clone());
                        (path, fact)
                    })
                })
                .collect(),
        }
    }
}
