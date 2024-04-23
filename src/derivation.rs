use crate::ir::*;

#[derive(Debug, Clone)]
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
    fn replace(&self, path: &[&str], subtree: &Tree) -> Option<Tree> {
        match path.last() {
            Some(name) => match self {
                Tree::Step {
                    label,
                    consequent,
                    antecedents,
                } => Some(Tree::Step {
                    label: label.clone(),
                    consequent: consequent.clone(),
                    antecedents: {
                        let mut ret = vec![];
                        let mut count = 0;
                        for (n, t) in antecedents {
                            if n == name {
                                ret.push((n.clone(), t.replace(&path[..path.len() - 1], subtree)?));
                                count += 1;
                            } else {
                                ret.push((n.clone(), t.clone()));
                            }
                        }
                        if count == 0 {
                            return None;
                        }

                        if count > 1 {
                            panic!();
                        }

                        ret
                    },
                }),
                _ => None,
            },
            None => Some(subtree.clone()),
        }
    }

    // fn replace_(&mut self, mut path: Path, subtree: Tree) {
    //     match path.pop() {
    //         Some(name) => match self {
    //             Tree::Step { antecedents, .. } => antecedents
    //                 .iter_mut()
    //                 .find_map(|(n, t)| if name == *n { Some(t) } else { None })
    //                 .unwrap()
    //                 .replace_(path, subtree),
    //             _ => panic!(),
    //         },
    //         None => *self = subtree,
    //     }
    // }

    // fn goals(&self) -> Vec<(Path, &Fact)> {
    //     match self {
    //         Tree::Axiom(_) => vec![],
    //         Tree::Goal(fact) => vec![(vec![], fact)],
    //         Tree::Step {
    //             antecedents, label, ..
    //         } => antecedents
    //             .iter()
    //             .flat_map(|(_, subtree)| {
    //                 subtree.goals().into_iter().map(|(mut path, fact)| {
    //                     path.push(label.clone());
    //                     (path, fact)
    //                 })
    //             })
    //             .collect(),
    //     }
    // }
}
