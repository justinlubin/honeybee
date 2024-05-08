use crate::synthesis;

pub fn auto(options: Vec<synthesis::GoalOption>) -> synthesis::Choice {
    synthesis::Choice {
        goal: 0,
        computation: 0,
        assignment: 0,
    }
}
