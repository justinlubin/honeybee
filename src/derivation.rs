use crate::ir::*;

type Path<'a> = Vec<&'a str>;

enum Tree<'a> {
    Axiom(&'a Fact),
    Goal(&'a Fact),
    Step {
        label: &'a str,
        antecedents: Vec<(&'a str, Tree<'a>)>,
        consequent: &'a Fact,
    },
}

impl<'a> Tree<'a> {
    fn replace(&mut self, mut path: Path, subtree: Tree<'a>) {
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
                        path.push(label);
                        (path, fact)
                    })
                })
                .collect(),
        }
    }
}
