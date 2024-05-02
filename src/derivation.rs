use crate::ir::*;

#[derive(Debug, Clone)]
pub enum Tree {
    Axiom(Fact),
    Collector(FactName),
    Goal(FactName),
    // Same as ComputationSignature, but:
    // (i) facts are instantiated
    // (ii) recursively references Tree
    Step {
        label: String,
        antecedents: Vec<(String, Tree)>,
        consequent: Fact,
        side_condition: Predicate,
    },
}

impl Tree {
    fn from_query(q: &Query) -> Option<Tree> {
        if !q.closed() {
            return None;
        }

        Some(Tree::Step {
            label: q.computation_signature.name.clone(),
            antecedents: q
                .computation_signature
                .params
                .iter()
                .map(|(n, f, _)| (n.clone(), Tree::Goal(f.clone())))
                .collect(),
            consequent: Fact {
                name: q.fact_signature.name.clone(),
                args: vec![],
            },
            side_condition: q.computation_signature.precondition.clone(),
        })
    }

    pub fn new(top_level_goal: &Fact) -> Tree {
        Tree::from_query(&Query::from_fact(top_level_goal)).unwrap()
    }

    pub fn replace(&self, path: &[&str], subtree: &Tree) -> Option<Tree> {
        match path.last() {
            Some(name) => match self {
                Tree::Step {
                    label,
                    consequent,
                    side_condition,
                    antecedents,
                } => Some(Tree::Step {
                    label: label.clone(),
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

    pub fn queries(&self, lib: &Library) -> Vec<(Vec<String>, Query)> {
        match self {
            Tree::Step {
                antecedents,
                side_condition,
                consequent,
                ..
            } => {
                let mut goal_siblings = vec![];
                let mut ret = vec![];

                for (n, t) in antecedents {
                    match t {
                        Tree::Axiom(..) => (),
                        Tree::Collector(_) => (),
                        Tree::Goal(q) => {
                            goal_siblings.push((n.clone(), q.clone()))
                        }
                        Tree::Step { .. } => {
                            for (mut path, q) in t.queries(lib) {
                                path.push(n.clone());
                                ret.push((path, q))
                            }
                        }
                    }
                }

                if !goal_siblings.is_empty() {
                    ret.push((
                        vec![],
                        Query::free(
                            lib,
                            goal_siblings,
                            side_condition
                                .iter()
                                .map(|pr| {
                                    pr.substitute_all(
                                        consequent
                                            .args
                                            .iter()
                                            .map(|(n, v)| {
                                                (n.as_str(), Query::RET, v)
                                            })
                                            .collect(),
                                    )
                                })
                                .collect(),
                        ),
                    ))
                };

                ret
            }
            _ => vec![],
        }
    }

    fn make_dashes(amount: usize) -> String {
        std::iter::repeat('-').take(amount * 2).collect()
    }

    fn _fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        depth: usize,
        prefix: &str,
    ) -> std::fmt::Result {
        match self {
            Tree::Axiom(fact) => write!(
                f,
                "{} {}{} [&axiom]",
                Tree::make_dashes(depth),
                prefix,
                crate::syntax::unparse::fact(fact),
            ),
            Tree::Collector(fact_name) => write!(
                f,
                "{} {}{} [&collector]",
                Tree::make_dashes(depth),
                prefix,
                fact_name,
            ),
            Tree::Goal(fact_name) => write!(
                f,
                "{} {}*** {}",
                Tree::make_dashes(depth),
                prefix,
                fact_name
            ),
            Tree::Step {
                label,
                antecedents,
                consequent,
                side_condition,
            } => {
                write!(
                    f,
                    "{} {}{} [{}] {}",
                    Tree::make_dashes(depth),
                    prefix,
                    crate::syntax::unparse::fact(consequent),
                    label,
                    crate::syntax::unparse::predicate(side_condition)
                        .replace("\n", ""),
                )?;
                for (n, t) in antecedents {
                    write!(f, "\n")?;
                    t._fmt(f, depth + 1, &format!("<{}>: ", n))?;
                }
                Ok(())
            }
        }
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self._fmt(f, 1, "")
    }
}
