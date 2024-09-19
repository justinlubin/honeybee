use crate::derivation;

use crate::ir::*;

enum Mode<'a> {
    Any,
    All,
    Particular(&'a derivation::Tree),
}

fn enumerate(
    lib: &Library,
    prog: &Program,
    mode: Mode,
) -> Vec<derivation::Tree> {
    let mut trees = vec![derivation::Tree::from_goal(&prog.goal)];
    loop {
        let t = trees.pop();
        todo!()
    }
}

pub fn enumerate_any(
    lib: &Library,
    prog: &Program,
) -> Option<derivation::Tree> {
    enumerate(lib, prog, Mode::Any).pop()
}

pub fn enumerate_all(lib: &Library, prog: &Program) -> Vec<derivation::Tree> {
    enumerate(lib, prog, Mode::All)
}

pub fn enumerate_particular(
    lib: &Library,
    prog: &Program,
    particular: &derivation::Tree,
) -> Option<derivation::Tree> {
    enumerate(lib, prog, Mode::Particular(particular)).pop()
}
