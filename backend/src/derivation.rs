use crate::ir::*;

#[derive(Debug, Clone)]
pub struct PathEntry {
    pub computation: String,
    pub tag: String,
}

pub fn computations(path: &Vec<PathEntry>) -> Vec<String> {
    path.iter()
        .map(|PathEntry { computation, .. }| computation.clone())
        .collect()
}

pub fn into_tags(path: Vec<PathEntry>) -> Vec<String> {
    path.into_iter().map(|PathEntry { tag, .. }| tag).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Tree {
    Axiom(Fact),
    Goal(FactName),
    Collect(FactName, Option<Vec<Fact>>),
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
    pub fn from_computation_signature(
        cs: &ComputationSignature,
        ret_args: Vec<(String, Value)>,
    ) -> Tree {
        Tree::Step {
            label: cs.name.clone(),
            antecedents: cs
                .params
                .iter()
                .map(|(tag, fact_name, mode)| {
                    (
                        tag.clone(),
                        match mode {
                            Mode::Exists => Tree::Goal(fact_name.clone()),
                            Mode::ForAll | Mode::ForAllPlus => {
                                Tree::Collect(fact_name.clone(), None)
                            }
                        },
                    )
                })
                .collect(),
            consequent: Fact {
                name: cs.ret.clone(),
                args: ret_args,
            },
            side_condition: cs.precondition.clone(),
        }
    }

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
                .filter_map(|(tag, fact_name, mode)| match mode {
                    Mode::Exists => {
                        Some((tag.clone(), Tree::Goal(fact_name.clone())))
                    }
                    Mode::ForAll | Mode::ForAllPlus => None,
                })
                .collect(),
            consequent: Fact {
                name: q.fact_signature.name.clone(),
                args: vec![],
            },
            side_condition: q.computation_signature.precondition.clone(),
        })
    }

    pub fn from_goal(top_level_goal: &Fact) -> Tree {
        Tree::from_query(&Query::from_fact(top_level_goal, "output")).unwrap()
    }

    pub fn replace(&self, path: &[String], subtree: &Tree) -> Tree {
        match path.first() {
            Some(name) => match self {
                Tree::Step {
                    label,
                    consequent,
                    side_condition,
                    antecedents,
                } => Tree::Step {
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
                                    t.replace(&path[1..], subtree),
                                ));
                                count += 1;
                            } else {
                                ret.push((n.clone(), t.clone()));
                            }
                        }
                        if count == 0 {
                            panic!("Selector name not found: {}", name)
                        }
                        ret
                    },
                },
                _ => panic!("Path on non-step"),
            },
            None => subtree.clone(),
        }
    }

    pub fn add_side_condition(
        &self,
        path: &[String],
        additional_condition: &Predicate,
    ) -> Tree {
        match path.first() {
            Some(name) => match self {
                Tree::Step {
                    label,
                    antecedents,
                    consequent,
                    side_condition,
                } => Tree::Step {
                    label: label.clone(),
                    antecedents: antecedents
                        .iter()
                        .map(|(x, t)| {
                            if x == name {
                                (
                                    x.clone(),
                                    t.add_side_condition(
                                        &path[..path.len() - 1],
                                        additional_condition,
                                    ),
                                )
                            } else {
                                (x.clone(), t.clone())
                            }
                        })
                        .collect(),
                    consequent: consequent.clone(),
                    side_condition: side_condition.clone(),
                },
                _ => panic!("Invalid path for tree"),
            },
            None => match self {
                Tree::Step {
                    label,
                    antecedents,
                    consequent,
                    side_condition,
                } => Tree::Step {
                    label: label.clone(),
                    antecedents: antecedents.clone(),
                    consequent: consequent.clone(),
                    side_condition: side_condition
                        .iter()
                        .cloned()
                        .chain(additional_condition.clone())
                        .collect(),
                },
                _ => panic!("Invalid path for tree (ends in non-step)"),
            },
        }
    }

    pub fn queries(&self, lib: &Library) -> Vec<(Vec<PathEntry>, Query)> {
        // TODO: will need to do something like this to fill the Collects
        // at program construction time (only do on complete trees?)
        match self {
            Tree::Step {
                antecedents,
                side_condition,
                consequent,
                ..
            } => {
                let mut axiom_equalities = vec![];
                let mut goal_siblings = vec![];
                let mut ret = vec![];

                for (tag, t) in antecedents {
                    match t {
                        Tree::Axiom(fact) => {
                            for (n, v) in &fact.args {
                                axiom_equalities.push(
                                    PredicateRelation::BinOp(
                                        PredicateRelationBinOp::Eq,
                                        PredicateAtom::Select {
                                            selector: n.clone(),
                                            arg: tag.clone(),
                                        },
                                        PredicateAtom::Const(v.clone()),
                                    ),
                                );
                            }
                        }
                        Tree::Collect(..) => (),
                        Tree::Goal(q) => {
                            goal_siblings.push((tag.clone(), q.clone()))
                        }
                        Tree::Step { label, .. } => {
                            for (mut path, q) in t.queries(lib) {
                                // TODO: possibly inefficient
                                path.insert(
                                    0,
                                    PathEntry {
                                        computation: label.clone(),
                                        tag: tag.clone(),
                                    },
                                );
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
                                    // TODO: probably don't need to substitute; just add more
                                    // equality constraints?
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
                                .chain(axiom_equalities)
                                .collect(),
                        ),
                    ))
                };

                ret
            }
            _ => vec![],
        }
    }

    pub fn complete(&self, including_collects: bool) -> bool {
        match self {
            Tree::Axiom(..) => true,
            Tree::Collect(_, facts_option) => {
                if including_collects {
                    facts_option.is_some()
                } else {
                    true
                }
            }
            Tree::Goal(..) => false,
            Tree::Step { antecedents, .. } => {
                for (_, t) in antecedents {
                    if !t.complete(including_collects) {
                        return false;
                    }
                }
                true
            }
        }
    }

    #[allow(unreachable_code)]
    pub fn collect(&mut self) {
        if !self.complete(false) {
            panic!("Can only collect on complete tree");
        }

        match self {
            Tree::Step { antecedents, .. } => {
                for i in 0..antecedents.len() {
                    match &antecedents[i].1 {
                        Tree::Collect(_, Some(_)) => {
                            panic!("Already collected tree")
                        }
                        Tree::Collect(fact_name, None) => {
                            antecedents[i].1 =
                                Tree::Collect(fact_name.clone(), Some(todo!()))
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    pub fn postorder(&self) -> Vec<(Vec<String>, &Tree)> {
        let mut ret = vec![];
        match self {
            Tree::Axiom(..) | Tree::Collect(..) | Tree::Goal(..) => (),
            Tree::Step { antecedents, .. } => {
                for (tag, t) in antecedents {
                    for (mut path, tt) in t.postorder() {
                        path.insert(0, tag.clone());
                        ret.push((path, tt));
                    }
                }
            }
        }
        ret.push((vec![], self));
        ret
    }

    pub fn valid(&self, annotations: &Vec<Fact>) -> bool {
        match self {
            Tree::Axiom(f) => annotations.contains(f),
            Tree::Goal(_) => false,
            Tree::Collect(_, _) => todo!(),
            Tree::Step {
                antecedents,
                consequent,
                side_condition,
                ..
            } => {
                let mut antecedent_facts = vec![];
                for (tag, subtree) in antecedents {
                    if !subtree.valid(annotations) {
                        return false;
                    }
                    antecedent_facts.push((
                        tag,
                        match subtree {
                            Tree::Axiom(f) => f,
                            Tree::Goal(_) => unreachable!(),
                            Tree::Collect(_, _) => todo!(),
                            Tree::Step { consequent, .. } => consequent,
                        },
                    ));
                }
                side_condition
                    .iter()
                    .all(|atom| atom.sat(consequent, &antecedent_facts))
            }
        }
    }

    pub fn pretty(&self) -> termtree::Tree<String> {
        let mut gp = termtree::GlyphPalette::new();
        gp.item_indent = "─";
        gp.skip_indent = " ";

        use ansi_term::Color::*;
        gp.middle_item = Fixed(8).paint(gp.middle_item).to_string().leak();
        gp.last_item = Fixed(8).paint(gp.last_item).to_string().leak();
        gp.item_indent = Fixed(8).paint(gp.item_indent).to_string().leak();
        gp.middle_skip = Fixed(8).paint(gp.middle_skip).to_string().leak();
        gp.last_skip = Fixed(8).paint(gp.last_skip).to_string().leak();
        gp.skip_indent = Fixed(8).paint(gp.skip_indent).to_string().leak();

        // Comment out first branch to understand tree a bit better
        match self {
            Tree::Step {
                consequent,
                antecedents,
                ..
            } if consequent.name == Query::GOAL_FACT_NAME
                && antecedents.len() == 1 =>
            {
                antecedents[0]
                    .1
                    .termtree(gp, &format!("{}", antecedents[0].0))
            }
            _ => self.termtree(gp, ""),
        }
    }

    fn termtree(
        &self,
        gp: termtree::GlyphPalette,
        prefix: &str,
    ) -> termtree::Tree<String> {
        use crate::syntax::unparse;
        use ansi_term::Color::*;

        // termtree::Tree::new(Red.paint("hi!"))
        match self {
            Tree::Axiom(fact) => termtree::Tree::new(format!(
                "{} {} {} {}",
                Purple.paint("•"),
                Blue.paint(prefix),
                Purple.paint("[fact]"),
                Fixed(8).paint(unparse::fact(fact))
            ))
            .with_glyphs(gp),
            Tree::Collect(fact_name, _) => termtree::Tree::new(format!(
                "{} {} {} {}{}{}",
                Purple.paint("•"),
                Blue.paint(prefix),
                Purple.paint("[collect]"),
                Fixed(8).paint("("),
                Purple.paint(fact_name),
                Fixed(8).paint(")"),
            ))
            .with_glyphs(gp),
            Tree::Goal(fact_name) => termtree::Tree::new(format!(
                "{} {} {} {}{}{}",
                Yellow.paint("•"),
                Yellow.paint(prefix),
                Yellow.paint("[goal]"),
                Yellow.paint("("),
                Yellow.paint(fact_name),
                Yellow.paint(")"),
            ))
            .with_glyphs(gp),
            Tree::Step {
                label,
                antecedents,
                consequent,
                ..
            } => {
                let mut t = termtree::Tree::new(format!(
                    "{} {} {} {}",
                    Green.paint("•"),
                    Blue.paint(prefix),
                    Green.paint(format!("({})", label)),
                    Fixed(8).paint(unparse::fact(consequent))
                ))
                .with_glyphs(gp);
                for (tag, subtree) in antecedents {
                    t.push(subtree.termtree(gp, &format!("{}", tag)));
                }
                t
            }
        }
    }
}

impl std::fmt::Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.pretty())
    }
}
