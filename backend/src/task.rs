use crate::ir::*;

use crate::derivation;

use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Serialize)]
pub enum Task<'a> {
    AnyValid,
    AllValid,
    AnySimplyTyped,
    AllSimplyTyped,
    Particular(&'a derivation::Tree),
}

impl<'a> fmt::Display for Task<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Task::AnyValid => write!(f, "Any"),
            Task::AllValid => write!(f, "All"),
            Task::AnySimplyTyped => write!(f, "AnySimplyTyped"),
            Task::AllSimplyTyped => write!(f, "AllSimplyTyped"),
            Task::Particular(_) => write!(f, "Particular"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SynthesisProblem<'a> {
    pub lib: &'a Library,
    pub prog: &'a Program,
    pub task: Task<'a>,
    pub soft_timeout: u128, // milliseconds
}

#[derive(Debug, Clone)]
pub struct SynthesisResult {
    pub results: Vec<derivation::Tree>,
    pub completed: bool,
}
