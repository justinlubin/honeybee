use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Algorithm {
    E,
    EP,
    PBN_E,
    PBN_EP,
    PBN_DL,
    PBN_DLmem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Task {
    Any,
    All,
    Particular,
}

pub const ALGORITHMS: [Algorithm; 6] = [
    Algorithm::E,
    Algorithm::EP,
    Algorithm::PBN_E,
    Algorithm::PBN_EP,
    Algorithm::PBN_DL,
    Algorithm::PBN_DLmem,
];

pub const TASKS: [Task; 3] = [Task::Particular, Task::Any, Task::All];
