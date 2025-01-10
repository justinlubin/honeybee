use crate::ir::*;

use crate::derivation::*;
use crate::egglog_adapter;
use crate::enumerate;
use crate::task;

#[derive(Debug, Clone)]
pub struct Synthesizer<'a> {
    pub tree: Tree,
    lib: &'a Library,
    prog: &'a Program,
}

#[derive(Debug, Clone)]
pub struct ComputationOption {
    pub name: String,
    pub assignment_options: Vec<Assignment>,
}

#[derive(Debug, Clone)]
pub enum GoalOption {
    Output {
        path: Vec<PathEntry>,
        tag: String,
        computation_options: Vec<ComputationOption>,
    },
    Input {
        path: Vec<PathEntry>,
        tag: String,
        fact_name: String,
        assignment_options: Vec<Assignment>,
    },
}

// TODO: computation field ignored for annotation goals
#[derive(Debug, Clone)]
pub enum Choice {
    Output {
        path: Vec<String>,
        tag: String,
        computation_name: String,
        assignment: Assignment,
    },
    Input {
        path: Vec<String>,
        tag: String,
        fact_name: String,
        assignment: Assignment,
    },
}

fn restrict_one(assignment: &Assignment, var: &str) -> Vec<(String, Value)> {
    let mut ret_args = vec![];

    for (lhs, rhs) in assignment {
        let components = lhs
            .strip_prefix("fv%")
            .unwrap()
            .split("*")
            .collect::<Vec<&str>>();

        assert!(components.len() == 2);

        let arg = components[0].to_owned();
        let selector = components[1].to_owned();

        if arg == *var {
            ret_args.push((selector, rhs.clone()))
        }
    }

    ret_args
}

fn restrict(assignments: &Vec<Assignment>, var: &str) -> Vec<Assignment> {
    let var_substr = format!("fv%{}*", var);
    let mut new_assignments = vec![];
    for a in assignments {
        let mut new_a = vec![];
        for (k, v) in a {
            if k.contains(&var_substr) {
                new_a.push((k.clone(), v.clone()));
            }
        }
        new_a.sort();
        new_assignments.push(new_a);
    }
    new_assignments.sort();
    new_assignments.dedup();
    new_assignments
        .into_iter()
        .map(|kvs| kvs.into_iter().collect())
        .collect()
}

fn free_to_args(
    lib: &Library,
    params: &Vec<(String, String, Mode)>,
    a: &Vec<(String, Value)>,
) -> Vec<(String, Fact)> {
    let mut ret = vec![];

    for (param, fact_name, _) in params {
        let fact_sig = lib.fact_signature(&fact_name).unwrap();

        let mut args = vec![];
        for (selector, _) in &fact_sig.params {
            let key = format!("fv%{}*{}", param, selector);
            args.push((
                selector.clone(),
                a.iter()
                    .find_map(
                        |(k, v)| if key == *k { Some(v.clone()) } else { None },
                    )
                    .unwrap(),
            ));
        }
        ret.push((
            param.clone(),
            Fact {
                name: fact_name.clone(),
                args,
            },
        ));
    }

    ret
}

impl<'a> Synthesizer<'a> {
    pub fn new(lib: &'a Library, prog: &'a Program) -> Synthesizer<'a> {
        Synthesizer {
            tree: Tree::from_goal(&prog.goal),
            lib,
            prog,
        }
    }

    pub fn options_enumerative(
        &self,
        soft_timeout: u128,
        enumerate_config: enumerate::Config,
    ) -> Option<Vec<GoalOption>> {
        let mut ops = vec![];
        let start = std::time::Instant::now();

        for (path, query) in self.tree.queries(self.lib) {
            let mut query_support: Vec<Vec<(String, Value)>> =
                enumerate::support(&self.prog, &query.fact_signature.params);

            match enumerate_config {
                enumerate::Config::Basic => (),
                enumerate::Config::Prune => {
                    query_support = query_support
                        .into_iter()
                        .filter(|a| {
                            query.computation_signature.precondition.iter().all(
                                |pr| {
                                    pr.sat(
                                        &Fact {
                                            name: query
                                                .fact_signature
                                                .name
                                                .clone(),
                                            args: a.clone(),
                                        },
                                        &free_to_args(
                                            self.lib,
                                            &query.computation_signature.params,
                                            a,
                                        ),
                                    )
                                },
                            )
                        })
                        .collect();
                }
            }

            let mut basic_assignments: Vec<Assignment> = vec![];
            for query_args in &query_support {
                let elapsed = start.elapsed().as_millis();

                if elapsed > soft_timeout {
                    return None;
                }

                let t = Tree::from_computation_signature(
                    &query.computation_signature,
                    query_args.clone(),
                );
                let basic_worklist = vec![t];

                let task::SynthesisResult { completed, results } =
                    enumerate::synthesize_worklist(
                        task::SynthesisProblem {
                            lib: self.lib,
                            prog: self.prog,
                            task: task::Task::AnyValid,
                            soft_timeout: soft_timeout - elapsed,
                        },
                        enumerate_config.clone(),
                        basic_worklist,
                    );

                if !completed {
                    return None;
                }

                if !results.is_empty() {
                    basic_assignments
                        .push(query_args.iter().cloned().collect());
                }
            }

            for (cut_param, cut_fact_name, _mode) in
                &query.computation_signature.params
            {
                let cut_fact_sig =
                    self.lib.fact_signature(&cut_fact_name).unwrap();

                ops.push(match cut_fact_sig.kind {
                    FactKind::Input => GoalOption::Input {
                        fact_name: cut_fact_name.clone(),
                        path: path.clone(),
                        tag: cut_param.clone(),
                        assignment_options: restrict(
                            &basic_assignments,
                            &cut_param,
                        ),
                    },
                    FactKind::Output => {
                        let mut computation_options = vec![];
                        for lemma in self
                            .lib
                            .matching_computation_signatures(cut_fact_name)
                        {
                            let mut assignment_options: Vec<Assignment> =
                                vec![];

                            for query_args in &query_support {
                                let elapsed = start.elapsed().as_millis();

                                if elapsed > soft_timeout {
                                    return None;
                                }

                                let t = Tree::from_computation_signature(
                                    &query.computation_signature,
                                    query_args.clone(),
                                );

                                let lemma_ret_args: Vec<_> = query_args
                                    .iter()
                                    .filter_map(|(lhs, rhs)| {
                                        let components = lhs
                                            .strip_prefix("fv%")
                                            .unwrap()
                                            .split("*")
                                            .collect::<Vec<&str>>();

                                        assert!(components.len() == 2);

                                        let arg = components[0].to_owned();
                                        let selector = components[1].to_owned();

                                        if arg == *cut_param {
                                            Some((selector, rhs.clone()))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();

                                let worklist = vec![t.replace(
                                    &[cut_param.clone()],
                                    &Tree::from_computation_signature(
                                        lemma,
                                        lemma_ret_args,
                                    ),
                                )];

                                let task::SynthesisResult {
                                    completed,
                                    results,
                                } = enumerate::synthesize_worklist(
                                    task::SynthesisProblem {
                                        lib: self.lib,
                                        prog: self.prog,
                                        task: task::Task::AnyValid,
                                        soft_timeout: soft_timeout - elapsed,
                                    },
                                    enumerate_config.clone(),
                                    worklist,
                                );

                                if !completed {
                                    return None;
                                }

                                if !results.is_empty() {
                                    assignment_options.push(
                                        query_args.iter().cloned().collect(),
                                    );
                                }
                            }
                            if assignment_options.is_empty() {
                                continue;
                            }
                            computation_options.push(ComputationOption {
                                name: lemma.name.clone(),
                                assignment_options,
                            });
                        }

                        GoalOption::Output {
                            path: path.clone(),
                            tag: cut_param.clone(),
                            computation_options,
                        }
                    }
                });
            }
        }

        Some(ops)
    }

    // Precondition: egg must have same lib and prog as synthesizer
    pub fn options_datalog(
        &self,
        egg: &mut egglog_adapter::Instance,
    ) -> Vec<GoalOption> {
        let mut ops = vec![];

        for (path, query) in self.tree.queries(self.lib) {
            let basic_assignments = egg.query(&query);

            for (cut_param, cut_fact_name, _mode) in
                &query.computation_signature.params
            {
                ops.push(
                    match self.lib.fact_signature(cut_fact_name).unwrap().kind {
                        FactKind::Input => GoalOption::Input {
                            fact_name: cut_fact_name.clone(),
                            path: path.clone(),
                            tag: cut_param.clone(),
                            assignment_options: restrict(
                                &basic_assignments,
                                &cut_param,
                            ),
                        },
                        FactKind::Output => GoalOption::Output {
                            computation_options: self
                                .lib
                                .matching_computation_signatures(cut_fact_name)
                                .into_iter()
                                .filter_map(|lemma| {
                                    let assignment_options = egg.query(
                                        &query.cut(self.lib, cut_param, lemma),
                                    );
                                    if assignment_options.is_empty() {
                                        None
                                    } else {
                                        Some(ComputationOption {
                                            assignment_options: restrict(
                                                &assignment_options,
                                                &cut_param,
                                            ),
                                            name: lemma.name.clone(),
                                        })
                                    }
                                })
                                .collect(),
                            path: path.clone(),
                            tag: cut_param.clone(),
                        },
                    },
                )
            }
        }

        ops
    }

    // TODO it's possible that there is some bug because not adding to side
    // condition?
    pub fn step(&mut self, choice: &Choice) {
        match choice {
            Choice::Input {
                path,
                tag,
                fact_name,
                assignment,
            } => {
                let args = restrict_one(&assignment, &tag);

                self.tree = self
                    .tree
                    .replace(
                        &path
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tag.clone()))
                            .collect::<Vec<String>>(),
                        &Tree::Axiom(Fact {
                            name: fact_name.clone(),
                            args: args.clone(),
                        }),
                    )
                    .sub_side_condition(
                        path,
                        &args
                            .into_iter()
                            .map(|(s, v)| (s, tag.clone(), v))
                            .collect(),
                    )
            }
            Choice::Output {
                path,
                tag,
                computation_name,
                assignment,
            } => {
                let cs =
                    self.lib.computation_signature(&computation_name).unwrap();

                let args = restrict_one(&assignment, &tag);

                self.tree = self
                    .tree
                    .replace(
                        &path
                            .iter()
                            .cloned()
                            .chain(std::iter::once(tag.clone()))
                            .collect::<Vec<String>>(),
                        &Tree::from_computation_signature(cs, args.clone()),
                    )
                    .sub_side_condition(
                        path,
                        &args
                            .into_iter()
                            .map(|(s, v)| (s, tag.clone(), v))
                            .collect(),
                    )
            }
        }
    }
}
