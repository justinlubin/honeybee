use crate::ir::*;

#[derive(Debug, Clone)]
pub enum Tree {
    Axiom(Fact),
    Goal(BasicQuery),
    // Same as ComputationSignature, but:
    // (i) facts are instantiated
    // (ii) recursively references Tree
    Step {
        name: String,
        antecedents: Vec<(String, Tree)>,
        consequent: Fact,
        side_condition: Predicate,
    },
}

impl Tree {
    pub fn replace(&self, path: &[&str], subtree: &Tree) -> Option<Tree> {
        match path.last() {
            Some(name) => match self {
                Tree::Step {
                    name,
                    consequent,
                    side_condition,
                    antecedents,
                } => Some(Tree::Step {
                    name: name.clone(),
                    consequent: consequent.clone(),
                    side_condition: side_condition.clone(),
                    antecedents: {
                        let mut ret = vec![];
                        let mut count = 0;
                        for (n, t) in antecedents {
                            if n == name {
                                ret.push((
                                    n.clone(),
                                    t.replace(
                                        &path[..path.len() - 1],
                                        subtree,
                                    )?,
                                ));
                                count += 1;
                            } else {
                                ret.push((n.clone(), t.clone()));
                            }
                        }
                        if count == 0 {
                            return None;
                        }
                        ret
                    },
                }),
                _ => None,
            },
            None => Some(subtree.clone()),
        }
    }

    pub fn immediately_partial_steps(&self) -> Vec<(Vec<String>, Query)> {
        match self {
            Tree::Step {
                antecedents,
                side_condition,
                ..
            } => {
                let siblings: Vec<_> = antecedents
                    .iter()
                    .filter_map(|(n, t)| match t {
                        Tree::Goal(q) => Some((n.clone(), q.clone())),
                        _ => None,
                    })
                    .collect();

                let mine = if siblings.is_empty() {
                    None
                } else {
                    Some((
                        vec![],
                        Query {
                            entries: siblings,
                            side_condition: side_condition.clone(),
                        },
                    ))
                };

                mine.into_iter()
                    .chain(antecedents.iter().flat_map(|(n, t)| {
                        t.immediately_partial_steps().into_iter().map(
                            |(mut path, q)| {
                                path.push(n.clone());
                                (path, q)
                            },
                        )
                    }))
                    .collect::<Vec<(Vec<String>, Query)>>()
            }
            _ => vec![],
        }
    }
}
